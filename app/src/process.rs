// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::{bounded, Receiver, Sender};
use hex::encode_to_slice;
use imbl::{ordset, OrdSet};
use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, DirEntry, Metadata},
    io::{Result as IoResult, Write},
    num::NonZeroUsize,
    path::PathBuf,
    str::from_utf8_unchecked,
    thread::{self, JoinHandle},
};
use tinyvec::TinyVec;

use crate::{
    arguments::Args,
    common::{get_capacity, increment, Aborted, Digest, ExitStatus, Flag, TinyVecEx},
    digest::{compute_digest, Error as DigestError},
    environment::Env,
    io::{DataSource, Error as IoError, OutStream},
    os::{file_id, DevId, FileId, STDIN_NAME},
    print_error, print_warn,
    thread_pool::{detect_thread_count, Cancelled, TaskResult, ThreadPool},
};

type FsId = Option<DevId>;
type IdSet = OrdSet<FileId>;
type Count = NonZeroUsize;

// ---------------------------------------------------------------------------
// Error Type
// ---------------------------------------------------------------------------

/// Error type for processing file tasks
#[derive(Debug)]
enum Error {
    NotFound(PathBuf),
    WalkOpen(PathBuf),
    WalkRead(PathBuf),
    ObjIsDir(PathBuf),
    FileOpen(PathBuf),
    FileRead(PathBuf),
}

impl Error {
    #[inline]
    fn from_io_error(error: IoError, path: PathBuf) -> Self {
        match error {
            IoError::AccessDenied => Error::FileOpen(path),
            IoError::FileNotFound => Error::NotFound(path),
            IoError::IsADirectory => Error::ObjIsDir(path),
        }
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Get file type from a directory entry (will resolve symlinks, if necessary)
#[inline]
fn get_metadata(dir_entry: &DirEntry) -> Option<Metadata> {
    match dir_entry.metadata() {
        Ok(meta_data) => match meta_data.is_symlink() {
            false => Some(meta_data),
            true => fs::metadata(dir_entry.path()).ok(),
        },
        Err(_) => None,
    }
}

/// Appends a directory id to the set of visited directories
#[inline]
fn append(visited: &'_ IdSet, file_id: Option<FileId>) -> Cow<'_, IdSet> {
    file_id.map_or(Cow::Borrowed(visited), |uid| Cow::Owned(visited.update(uid)))
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

/// Compute the exit status
#[inline]
fn exit_status(write_errors: bool, file_errors: u64, args: &Args) -> ExitStatus {
    if (!write_errors) && (file_errors == u64::MIN) {
        ExitStatus::Success
    } else if (!write_errors) && ((file_errors == u64::MIN) || args.keep_going) {
        ExitStatus::Warning
    } else {
        ExitStatus::Failure
    }
}

// ---------------------------------------------------------------------------
// Print results
// ---------------------------------------------------------------------------

/// Print a single digest
#[inline]
fn print_digest(output: &mut dyn Write, file_name: &OsStr, digest: &Digest, args: &Args) -> IoResult<()> {
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
fn print_result(output: &mut OutStream, digest_result: &DigestResult, args: &Args) -> bool {
    match digest_result {
        Ok(digest) => print_digest(output.out(), digest.1.as_os_str(), &digest.0, args).is_ok(),
        Err(error) => {
            match error {
                Error::FileOpen(path) => print_error!(output, args, "Failed to open input file: {:?}", path),
                Error::FileRead(path) => print_error!(output, args, "Failed to read input file: {:?}", path),
                Error::NotFound(path) => print_error!(output, args, "Input file not found: {:?}", path),
                Error::ObjIsDir(path) => print_error!(output, args, "Input file is a directory: {:?}", path),
                Error::WalkOpen(path) => print_error!(output, args, "Failed to open directory: {:?}", path),
                Error::WalkRead(path) => print_error!(output, args, "Failed to read directory: {:?}", path),
            }
            true
        }
    }
}

/// Print the summary
#[inline]
fn print_summary(output: &mut OutStream, file_errors: u64, args: &Args) {
    if file_errors > u64::MIN {
        if args.keep_going {
            print_warn!(output, args, "Warning: {} file(s) were skipped due to errors!", file_errors);
        } else {
            print_error!(output, args, "Error: The checksum computation has failed!");
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
        Err(error) => Ok(Err(Error::from_io_error(error, file_name))),
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
fn do_iterate(path_tx: &Sender<PathResult>, dir_name: PathBuf, fs_id: FsId, visited: &IdSet, bfs: bool, args: &Args, halt: &Flag) -> Result<bool, Cancelled> {
    let dir_iter = match fs::read_dir(&dir_name) {
        Ok(dir_iter) => dir_iter,
        Err(_) => {
            path_tx.send(Err(Error::WalkOpen(dir_name.to_path_buf())))?;
            return Ok(false);
        }
    };

    let mut dir_queue: TinyVec<[_; 96usize]> = TinyVec::new();

    for element in dir_iter {
        match element {
            Ok(dir_entry) => {
                check_cancelled!(halt);
                let meta_data = get_metadata(&dir_entry);
                if meta_data.as_ref().is_some_and(|meta| meta.is_dir()) {
                    if args.recursive {
                        let unique_id = file_id(unsafe { meta_data.unwrap_unchecked() });
                        if unique_id.is_none_or(|uid| (args.cross_dev || fs_id.is_none_or(|dev| uid.same_dev(dev))) && !visited.contains(&uid)) {
                            if bfs {
                                dir_queue.push((unique_id, dir_entry.path()));
                            } else if !(do_iterate(path_tx, dir_entry.path(), fs_id, &append(visited, unique_id), bfs, args, halt)? || args.keep_going) {
                                return Ok(false);
                            }
                        }
                    }
                } else if args.all || meta_data.is_none_or(|meta| meta.is_file()) {
                    path_tx.send(Ok(dir_entry.path()))?;
                }
            }
            Err(_) => {
                path_tx.send(Err(Error::WalkRead(dir_name)))?;
                return Ok(false);
            }
        }
    }

    for (unique_id, dir_name) in dir_queue.into_iter() {
        check_cancelled!(halt);
        if !(do_iterate(path_tx, dir_name, fs_id, &append(visited, unique_id), bfs, args, halt)? || args.keep_going) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Iterate a list of input files
fn iterate_thread(path_tx: &Sender<PathResult>, bfs: bool, args: &Args, halt: &Flag) -> TaskResult {
    for file_name in args.files.iter().cloned() {
        check_cancelled!(halt);
        let directory = if args.dirs { fs::metadata(&file_name).ok().filter(|meta| meta.is_dir()) } else { None };
        if let Some(meta_data) = directory {
            let (visited, fs_id) = file_id(meta_data).map_or_else(Default::default, |uid| (ordset![uid], Some(uid.dev())));
            if !(do_iterate(path_tx, file_name, fs_id, &visited, bfs, args, halt)? || args.keep_going) {
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
fn start_iteration(bfs: bool, args: &'static Args, halt: &'static Flag) -> (Receiver<PathResult>, Option<JoinHandle<TaskResult>>) {
    if args.dirs || (args.files.len() > 1024usize) {
        let (path_tx, path_rx) = bounded::<PathResult>(256usize);
        (path_rx, Some(thread::spawn(move || iterate_thread(&path_tx, bfs, args, halt))))
    } else {
        let (path_tx, path_rx) = bounded::<PathResult>(args.files.len());
        args.files.iter().cloned().for_each(|path| path_tx.try_send(Ok(path)).unwrap());
        (path_rx, None)
    }
}

fn process_mt(output: &mut OutStream, n_threads: Count, out_size: usize, bfs: bool, args: &'static Args, halt: &'static Flag) -> Result<ExitStatus, Aborted> {
    // Initialize channel
    let (digest_tx, digest_rx) = bounded::<DigestResult>(get_capacity(&n_threads));

    // Start the file iteration thread
    let (path_rx, thread_handle) = start_iteration(bfs, args, halt);

    // Start the worker threads
    let thread_pool = ThreadPool::new(n_threads, move || compute_thread(&path_rx, &digest_tx, out_size, args, halt));

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
    print_summary(output, file_errors, args);

    // Check for errors
    Ok(exit_status(write_errors, file_errors, args))
}

fn process_st(output: &mut OutStream, out_size: usize, bfs: bool, args: &'static Args, halt: &'static Flag) -> Result<ExitStatus, Aborted> {
    // Start the file iteration thread
    let (path_rx, thread_handle) = start_iteration(bfs, args, halt);

    // Initialize counters
    let (mut file_errors, mut write_errors) = (u64::MIN, false);

    // Process all files in the queue
    while let Ok(path_result) = path_rx.recv() {
        break_cancelled!(halt);
        let digest_result = match path_result {
            Ok(path) => match compute_file_digest(path, out_size, args, halt) {
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
    print_summary(output, file_errors, args);

    // Check for errors
    Ok(exit_status(write_errors, file_errors, args))
}

// ---------------------------------------------------------------------------
// Process files
// ---------------------------------------------------------------------------

/// Process data from 'stdin' stream
fn process_stdin(output: &mut OutStream, digest_size: usize, args: &Args, halt: &Flag) -> Result<ExitStatus, Cancelled> {
    let mut stdin = DataSource::from_stdin();
    let mut digest = TinyVec::with_length(digest_size);

    match compute_digest(&mut stdin, digest.as_mut_slice(), args, halt) {
        Ok(_) => match print_digest(output.out(), &STDIN_NAME, &digest, args) {
            Ok(_) => Ok(ExitStatus::Success),
            Err(_) => Ok(ExitStatus::Failure),
        },
        Err(DigestError::IoError) => {
            print_error!(output, args, "Failed to read data from the standard input stream!");
            Ok(ExitStatus::Failure)
        }
        Err(DigestError::Cancelled) => Err(Cancelled),
    }
}

/// Process all input files
pub fn process_files(output: &mut OutStream, digest_size: usize, args: &'static Args, env: &Env, halt: &'static Flag) -> Result<ExitStatus, Aborted> {
    // Read input datat from 'stdin' stream?
    if args.files.is_empty() {
        return process_stdin(output, digest_size, args, halt).map_err(|_| Aborted);
    }

    // Determine number of threads
    let thread_count = detect_thread_count(args, env);

    // Determine directory walking strategy
    let breadth_first = env.dirwalk_strategy.unwrap_or(true);

    // Check if process has been aborted
    if !halt.running() {
        return Err(Aborted);
    }

    if thread_count > Count::MIN {
        process_mt(output, thread_count, digest_size, breadth_first, args, halt)
    } else {
        process_st(output, digest_size, breadth_first, args, halt)
    }
}
