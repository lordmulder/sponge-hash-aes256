// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::{bounded, Receiver, Sender};
use hex::encode_to_slice;
use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsStr,
    fs::{self, DirEntry, Metadata},
    io::{Result as IoResult, Write},
    iter,
    num::NonZeroUsize,
    path::PathBuf,
    str::from_utf8_unchecked,
    sync::Arc,
    thread::{self, JoinHandle},
};
use tinyvec::TinyVec;

use crate::{
    arguments::Args,
    common::{get_capacity, increment, Aborted, Digest, Flag, TinyVecEx},
    digest::{compute_digest, Error as DigestError},
    environment::Env,
    io::{DataSource, Error as IoError, STDIN_NAME},
    print_error,
    thread_pool::{detect_thread_count, Cancelled, TaskResult, ThreadPool},
};

// ---------------------------------------------------------------------------
// Error Type
// ---------------------------------------------------------------------------

/// Error type for processing file tasks
#[derive(Debug)]
#[allow(dead_code)]
enum Error {
    NotFound(PathBuf),
    WalkOpen(PathBuf),
    WalkRead(PathBuf),
    ObjIsDir(PathBuf),
    FileOpen(PathBuf),
    FileRead(PathBuf),
}

// ---------------------------------------------------------------------------
// Platform support
// ---------------------------------------------------------------------------

type FileId = (u64, u64);
type FileIdSet = BTreeSet<FileId>;

#[cfg(target_family = "unix")]
mod file_id {
    use super::*;
    use std::os::unix::fs::MetadataExt;

    /// Get the unique file id
    #[inline(always)]
    pub fn get(meta: &Metadata) -> Option<FileId> {
        Some((meta.dev(), meta.ino()))
    }
}

#[cfg(not(target_family = "unix"))]
mod file_id {
    use super::*;

    #[inline(always)]
    pub fn get(_: &Metadata) -> Option<FileId> {
        None
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Check if a directory entry is a directory (or a symlink to a directory)
#[inline]
fn is_directory(dir_entry: &DirEntry) -> Option<Metadata> {
    match dir_entry.metadata() {
        Ok(meta_data) => {
            let file_type = meta_data.file_type();
            match file_type.is_dir() {
                false => match file_type.is_symlink() {
                    true => fs::metadata(dir_entry.path()).ok().filter(|value| value.is_dir()),
                    false => None,
                },
                true => Some(meta_data),
            }
        }
        Err(_) => None,
    }
}

/// Appends a directory id to the set of visited directories
#[inline]
fn append(visited: &'_ FileIdSet, file_id: Option<FileId>) -> Cow<'_, FileIdSet> {
    file_id.map_or(Cow::Borrowed(visited), |id| {
        let mut cloned = visited.clone();
        cloned.insert(id);
        Cow::Owned(cloned)
    })
}

/// Check if the computation has been cancelled
macro_rules! check_cancelled {
    ($halt:ident) => {
        if !$halt.running() {
            return Err(Cancelled);
        }
    };
}

/// Check if the computation has been cancelled
macro_rules! break_cancelled {
    ($halt:ident) => {
        if !$halt.running() {
            break;
        }
    };
}

// ---------------------------------------------------------------------------
// Print results
// ---------------------------------------------------------------------------

/// Print a single digest
#[inline]
fn print_digest(output: &mut impl Write, file_name: &OsStr, digest: &Digest, args: &Args) -> IoResult<()> {
    let hex_length = digest.len().checked_mul(2usize).unwrap();
    let mut hex_buffer: TinyVec<[u8; 2usize * DEFAULT_DIGEST_SIZE]> = TinyVec::with_length(hex_length);

    encode_to_slice(digest.as_slice(), hex_buffer.as_mut_slice()).unwrap();
    let hex_string = unsafe { from_utf8_unchecked(hex_buffer.as_slice()) };

    if args.null {
        if args.plain {
            write!(output, "{}\0", hex_string)?;
        } else {
            write!(output, "{} {}\0", hex_string, file_name.to_string_lossy())?;
        }
    } else if args.plain {
        writeln!(output, "{}", hex_string)?;
    } else {
        writeln!(output, "{} {}", hex_string, file_name.to_string_lossy())?;
    }

    if args.flush {
        output.flush()?;
    }

    Ok(())
}

/// Print result to output
#[inline]
fn print_result(output: &mut impl Write, digest_result: &DigestResult, args: &Args) -> bool {
    match digest_result {
        Ok(digest) => print_digest(output, digest.1.as_os_str(), &digest.0, args).is_ok(),
        Err(error) => {
            match error {
                Error::NotFound(path) => print_error!(args, "Input file not found: {:?}", path),
                Error::FileOpen(path) => print_error!(args, "Failed to open input file: {:?}", path),
                Error::FileRead(path) => print_error!(args, "Failed to read input file: {:?}", path),
                Error::ObjIsDir(path) => print_error!(args, "Input file is a directory: {:?}", path),
                Error::WalkOpen(path) => print_error!(args, "Failed to open directory: {:?}", path),
                Error::WalkRead(path) => print_error!(args, "Failed to read directory: {:?}", path),
            }
            true
        }
    }
}

/// Print the summary
fn print_summary(file_errors: u64, args: &Args) {
    if file_errors > u64::MIN {
        if args.keep_going {
            print_error!(args, "WARNING: {} file(s) were skipped due to errors!", file_errors);
        } else {
            print_error!(args, "WARNING: The process failed with an error!");
        }
    }
}

// ---------------------------------------------------------------------------
// Compute file digest
// ---------------------------------------------------------------------------

type DigestResult = Result<(Digest, PathBuf), Error>;

fn compute_file_digest(file_name: PathBuf, digest_size: usize, args: &Args, halt: &Flag) -> Result<DigestResult, Cancelled> {
    match DataSource::from_path(&file_name) {
        Ok(mut source) => {
            let mut digest = TinyVec::with_length(digest_size);
            match compute_digest(&mut source, digest.as_mut_slice(), args, halt) {
                Ok(_) => Ok(Ok((digest, file_name))),
                Err(DigestError::IoError) => Ok(Err(Error::FileRead(file_name))),
                Err(DigestError::Cancelled) => Err(Cancelled),
            }
        }
        Err(error) => match error {
            IoError::FileNotFound => Ok(Err(Error::NotFound(file_name))),
            IoError::IsADirectory => Ok(Err(Error::ObjIsDir(file_name))),
            _ => Ok(Err(Error::FileOpen(file_name))),
        },
    }
}

fn compute_thread(path_rx: &Receiver<PathResult>, digest_tx: &Sender<DigestResult>, digest_size: usize, args: &Args, halt: &Flag) -> TaskResult {
    while let Ok(path_result) = path_rx.recv() {
        check_cancelled!(halt);
        match path_result {
            Ok(path) => {
                let digest_result = compute_file_digest(path, digest_size, args, halt).or(Err(Cancelled))?;
                let is_success = digest_result.is_ok();
                digest_tx.send(digest_result)?;
                if !(is_success || args.keep_going) {
                    break;
                }
            }
            Err(error) => digest_tx.send(Err(error))?,
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Iterate input files/directories
// ---------------------------------------------------------------------------

type PathResult = Result<PathBuf, Error>;

/// Iterate all files and sub-directories in a directory
fn iterate_directory(path_tx: &Sender<PathResult>, dir_name: PathBuf, visited: &FileIdSet, bfs: bool, args: &Args, halt: &Flag) -> Result<bool, Cancelled> {
    let dir_iter = match fs::read_dir(&dir_name) {
        Ok(dir_iter) => dir_iter,
        Err(_) => {
            path_tx.send(Err(Error::WalkOpen(dir_name.to_path_buf())))?;
            return Ok(false);
        }
    };

    let mut dir_queue = if bfs { Vec::with_capacity(32usize) } else { Vec::new() };

    for element in dir_iter {
        match element {
            Ok(dir_entry) => {
                check_cancelled!(halt);
                if let Some(meta_data) = is_directory(&dir_entry) {
                    if args.recursive {
                        let file_id = file_id::get(&meta_data);
                        if file_id.is_none_or(|id| !visited.contains(&id)) {
                            if bfs {
                                dir_queue.push((file_id, dir_entry.path()));
                            } else if !(iterate_directory(path_tx, dir_entry.path(), &append(visited, file_id), bfs, args, halt)? || args.keep_going) {
                                return Ok(false);
                            }
                        }
                    }
                } else {
                    path_tx.send(Ok(dir_entry.path()))?;
                }
            }
            Err(_) => {
                path_tx.send(Err(Error::WalkRead(dir_name)))?;
                return Ok(false);
            }
        }
    }

    for (file_id, dir_name) in dir_queue.into_iter() {
        check_cancelled!(halt);
        if !(iterate_directory(path_tx, dir_name, &append(visited, file_id), bfs, args, halt)? || args.keep_going) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Iterate a list of input files
fn iterate_thread(path_tx: &Sender<PathResult>, bfs: bool, args: &Args, halt: &Flag) -> TaskResult {
    let handle_directories = args.dirs || args.recursive;

    for file_name in args.files.iter().cloned() {
        check_cancelled!(halt);
        let directory_info = if handle_directories { fs::metadata(&file_name).ok().filter(|meta| meta.is_dir()) } else { None };
        if let Some(meta_data) = directory_info {
            let visited = file_id::get(&meta_data).map_or_else(FileIdSet::new, |dir_id| iter::once(dir_id).collect());
            if !(iterate_directory(path_tx, file_name, &visited, bfs, args, halt)? || args.keep_going) {
                break;
            }
        } else {
            path_tx.send(Ok(file_name))?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Process implementation
// ---------------------------------------------------------------------------

/// Start the file iteration thread, if it is needed
fn start_iteration(bfs: bool, args: &Arc<Args>, halt: &Arc<Flag>) -> (Receiver<PathResult>, Option<JoinHandle<TaskResult>>) {
    if args.dirs || args.recursive || args.files.len() > 1024usize {
        let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
        let (path_tx, path_rx) = bounded::<PathResult>(256usize);
        (path_rx, Some(thread::spawn(move || iterate_thread(&path_tx, bfs, &args_cloned, &halt_cloned))))
    } else {
        let (path_tx, path_rx) = bounded::<PathResult>(args.files.len());
        args.files.iter().cloned().for_each(|path| path_tx.try_send(Ok(path)).unwrap());
        (path_rx, None)
    }
}

fn process_mt(output: &mut impl Write, thread_count: NonZeroUsize, digest_size: usize, bfs: bool, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Initialize channel
    let (digest_tx, digest_rx) = bounded::<DigestResult>(get_capacity(&thread_count));

    // Start the file iteration thread
    let (path_rx, thread_handle) = start_iteration(bfs, args, halt);

    // Start the worker threads
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    let thread_pool = ThreadPool::new(thread_count, move || compute_thread(&path_rx, &digest_tx, digest_size, &args_cloned, &halt_cloned));

    // Initialize counters
    let (mut file_errors, mut write_errors) = (u64::MIN, false);

    // Process all digest results
    while let Ok(digest_result) = digest_rx.recv() {
        break_cancelled!(halt);
        if digest_result.is_err() {
            increment(&mut file_errors);
        }

        if !print_result(output, &digest_result, args) {
            write_errors = true;
            break;
        } else if !(digest_result.is_ok() || args.keep_going) {
            break;
        }
    }

    // Send shutdown signal to still running threads
    drop(digest_rx);
    let is_aborted = halt.stop_process().is_err();

    // Wait until the thread has completed
    if let Some(Err(error)) = thread_handle.map(|handle| handle.join()) {
        panic!("Failed to join the worker thread: {error:?}")
    }

    // Wait until all thread-pool tasks have completed too
    if let Err(error) = thread_pool.join() {
        panic!("Failed to join the worker thread: {error:?}")
    }

    // Has the process been aborted?
    if is_aborted {
        return Err(Aborted);
    }

    // Print warning if any file(s) have been skipped
    print_summary(file_errors, args);

    // Check for errors
    Ok((file_errors == u64::MIN) && (!write_errors))
}

fn process_st(output: &mut impl Write, digest_size: usize, bfs: bool, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Start the file iteration thread
    let (path_rx, thread_handle) = start_iteration(bfs, args, halt);

    // Initialize counters
    let (mut file_errors, mut write_errors) = (u64::MIN, false);

    // Process all files in the queue
    while let Ok(path_result) = path_rx.recv() {
        break_cancelled!(halt);
        let digest_result = match path_result {
            Ok(path) => match compute_file_digest(path, digest_size, args, halt) {
                Ok(result) => result,
                Err(Cancelled) => break, /* cancelled */
            },
            Err(error) => Err(error),
        };

        if digest_result.is_err() {
            increment(&mut file_errors);
        }

        if !print_result(output, &digest_result, args) {
            write_errors = true;
            break;
        } else if !(digest_result.is_ok() || args.keep_going) {
            break;
        }
    }

    // Send shutdown signal to still running threads
    drop(path_rx);
    let is_aborted = halt.stop_process().is_err();

    // Wait until the thread has completed
    if let Some(Err(error)) = thread_handle.map(|handle| handle.join()) {
        panic!("Failed to join the worker thread: {error:?}")
    }

    // Has the process been aborted?
    if is_aborted {
        return Err(Aborted);
    }

    // Print warning if any file(s) have been skipped
    print_summary(file_errors, args);

    // Check for errors
    Ok((file_errors == u64::MIN) && (!write_errors))
}

// ---------------------------------------------------------------------------
// Process files
// ---------------------------------------------------------------------------

/// Process data from 'stdin' stream
fn process_stdin(output: &mut impl Write, digest_size: usize, args: Arc<Args>, halt: Arc<Flag>) -> Result<bool, Cancelled> {
    let mut stdin = match DataSource::from_stdin() {
        Ok(stream) => stream,
        Err(_) => {
            print_error!(args, "Failed to acquire the standard input stream for reading!");
            return Ok(false);
        }
    };

    let mut digest = TinyVec::with_length(digest_size);

    match compute_digest(&mut stdin, digest.as_mut_slice(), &args, &halt) {
        Ok(_) => Ok(print_digest(output, &STDIN_NAME, &digest, &args).is_ok()),
        Err(DigestError::IoError) => {
            print_error!(args, "Failed to read data from the standard input stream!");
            Ok(false)
        }
        Err(DigestError::Cancelled) => Err(Cancelled),
    }
}

/// Process all input files
pub fn process_files(output: &mut impl Write, digest_size: usize, args: Arc<Args>, env: &Env, halt: Arc<Flag>) -> Result<bool, Aborted> {
    // Read input datat from 'stdin' stream?
    if args.files.is_empty() {
        return process_stdin(output, digest_size, args, halt).map_err(|_| Aborted);
    }

    // Determine number of threads
    let thread_count = detect_thread_count(&args, env);

    // Determine directory walking strategy
    let breadth_first = env.dirwalk_strategy.unwrap_or(true);

    if thread_count > NonZeroUsize::MIN {
        process_mt(output, thread_count, digest_size, breadth_first, &args, &halt)
    } else {
        process_st(output, digest_size, breadth_first, &args, &halt)
    }
}
