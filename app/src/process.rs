// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::{bounded, Receiver, SendError, Sender};
use hex::encode_to_slice;
use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsStr,
    fs::{self, DirEntry, Metadata},
    io::{Result as IoResult, Write},
    iter,
    num::NonZeroUsize,
    path::PathBuf,
    str::from_utf8,
    sync::{atomic::Ordering, Arc},
    thread,
};

use crate::{
    arguments::Args,
    common::{hardware_concurrency, Aborted, Flag, MAX_DIGEST_SIZE},
    digest::{compute_digest, Error as DigestError},
    environment::{get_search_strategy, get_thread_count},
    io::{DataSource, STDIN_NAME},
    print_error,
};

// ---------------------------------------------------------------------------
// Error Type
// ---------------------------------------------------------------------------

/// Error type for processing file tasks
#[derive(Debug)]
#[allow(dead_code)]
enum TaskError {
    DirOpen(PathBuf),
    DirRead(PathBuf),
    SrcIsDir(PathBuf),
    FileOpen(PathBuf),
    FileRead(PathBuf),
}

/// Error type to signal that a thread was aborted
enum ThreadError {
    Aborted,
    SendErr,
}

impl<T> From<SendError<T>> for ThreadError {
    fn from(_: SendError<T>) -> Self {
        Self::SendErr
    }
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

/// Check if the computation has been aborted
macro_rules! check_cancelled {
    ($halt:ident) => {
        if $halt.load(Ordering::Relaxed) {
            return Err(ThreadError::Aborted);
        }
    };
    ($halt:ident, $aborted:ident) => {
        if $halt.load(Ordering::Relaxed) {
            $aborted = true;
            break;
        }
    };
}

// ---------------------------------------------------------------------------
// Print results
// ---------------------------------------------------------------------------

fn print_result(output: &mut impl Write, digest_result: &DigestResult, digest_size: usize, args: &Args) -> bool {
    match digest_result {
        Ok(digest) => print_digest(output, digest.1.as_os_str(), &digest.0, digest_size, args).is_ok(),
        Err(error) => {
            match error {
                TaskError::DirOpen(path) => print_error!(args, "Failed to open directory: {:?}", path),
                TaskError::DirRead(path) => print_error!(args, "Failed to read directory: {:?}", path),
                TaskError::SrcIsDir(path) => print_error!(args, "Input file is a directory: {:?}", path),
                TaskError::FileOpen(path) => print_error!(args, "Failed to open input file: {:?}", path),
                TaskError::FileRead(path) => print_error!(args, "Failed to read input file: {:?}", path),
            }
            true
        }
    }
}

fn print_digest(output: &mut impl Write, file_name: &OsStr, digest: &[u8; MAX_DIGEST_SIZE], digest_size: usize, args: &Args) -> IoResult<()> {
    let mut hex_buffer = [0u8; MAX_DIGEST_SIZE * 2usize];
    let hex_slice = &mut hex_buffer[..digest_size.checked_mul(2usize).unwrap()];

    encode_to_slice(&digest[..digest_size], hex_slice).unwrap();

    if args.null {
        if args.plain {
            write!(output, "{}\0", from_utf8(hex_slice).unwrap())?;
        } else {
            write!(output, "{} {}\0", from_utf8(hex_slice).unwrap(), file_name.to_string_lossy())?;
        }
    } else if args.plain {
        writeln!(output, "{}", from_utf8(hex_slice).unwrap())?;
    } else {
        writeln!(output, "{} {}", from_utf8(hex_slice).unwrap(), file_name.to_string_lossy())?;
    }

    if args.flush {
        output.flush()?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Compute file digest
// ---------------------------------------------------------------------------

type DigestResult = Result<([u8; MAX_DIGEST_SIZE], PathBuf), TaskError>;

fn compute_file_digest(file_name: PathBuf, digest_size: usize, args: &Args, halt: &Flag) -> Result<DigestResult, Aborted> {
    match DataSource::from_path(&file_name) {
        Ok(mut source) => {
            if source.is_directory() {
                Ok(Err(TaskError::SrcIsDir(file_name)))
            } else {
                let mut digest = [0u8; MAX_DIGEST_SIZE];
                match compute_digest(&mut source, &mut digest[..digest_size], args, halt) {
                    Ok(_) => Ok(Ok((digest, file_name))),
                    Err(DigestError::IoError) => Ok(Err(TaskError::FileRead(file_name))),
                    Err(DigestError::Aborted) => Err(Aborted),
                }
            }
        }
        Err(_) => Ok(Err(TaskError::FileOpen(file_name))),
    }
}

fn compute_thread(path_rx: &Receiver<PathResult>, digest_tx: &Sender<DigestResult>, digest_size: usize, args: &Args, halt: &Flag) -> Result<(), ThreadError> {
    while let Ok(path_result) = path_rx.recv() {
        check_cancelled!(halt);
        match path_result {
            Ok(path) => {
                let digest_result = compute_file_digest(path, digest_size, args, halt).or(Err(ThreadError::Aborted))?;
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

type PathResult = Result<PathBuf, TaskError>;

/// Iterate all files and sub-directories in a directory
fn iterate_directory(path_tx: &Sender<PathResult>, dir_name: PathBuf, visited: &FileIdSet, bfs: bool, args: &Args, halt: &Flag) -> Result<bool, ThreadError> {
    let dir_iter = match fs::read_dir(&dir_name) {
        Ok(dir_iter) => dir_iter,
        Err(_) => {
            path_tx.send(Err(TaskError::DirOpen(dir_name.to_path_buf())))?;
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
                path_tx.send(Err(TaskError::DirRead(dir_name)))?;
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
fn iterate_thread(path_tx: &Sender<PathResult>, bfs: bool, args: &Args, halt: &Flag) -> Result<(), ThreadError> {
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

fn process_mt(output: &mut impl Write, thread_count: usize, digest_size: usize, bfs: bool, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Initialize thread pool
    let mut thread_pool = Vec::with_capacity(thread_count.saturating_add(1usize));
    let (path_tx, path_rx) = bounded::<PathResult>(thread_count.saturating_mul(16usize));
    let (digest_tx, digest_rx) = bounded::<DigestResult>(thread_count);

    // Start the file iteration thread
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    thread_pool.push(thread::spawn(move || iterate_thread(&path_tx, bfs, &args_cloned, &halt_cloned)));

    // Start the worker threads
    for (path_rx, digest_tx) in iter::repeat_n(path_rx, thread_count).zip(iter::repeat_n(digest_tx, thread_count)) {
        let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
        thread_pool.push(thread::spawn(move || compute_thread(&path_rx, &digest_tx, digest_size, &args_cloned, &halt_cloned)));
    }

    // Process all digest results
    let (mut file_errors, mut write_errors, mut is_aborted) = (u64::MIN, false, false);
    while let Ok(digest_result) = digest_rx.recv() {
        check_cancelled!(halt, is_aborted);
        if digest_result.is_err() {
            file_errors = file_errors.saturating_add(1u64);
        }
        if !print_result(output, &digest_result, digest_size, args) {
            write_errors = true;
            break;
        } else if !(digest_result.is_ok() || args.keep_going) {
            break;
        }
    }

    // Wait until all threads have completed
    drop(digest_rx);
    for thread in thread_pool.drain(..) {
        if matches!(thread.join().expect("Failed to join worker thread!"), Err(ThreadError::Aborted)) {
            is_aborted = true;
        }
    }

    // Has the process been aborted?
    if is_aborted {
        return Err(Aborted);
    }

    // Print warning if any file(s) have been skipped
    if args.keep_going && (file_errors > 0u64) {
        print_error!(args, "WARNING: {} file(s) were skipped due to errors.", file_errors);
    }

    // Check for errors
    Ok((file_errors == u64::MIN) && (!write_errors))
}

fn process_st(output: &mut impl Write, digest_size: usize, bfs: bool, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Initialize thread pool
    let (path_tx, path_rx) = bounded::<PathResult>(32usize);

    // Start the file iteration thread
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    let thread_handle = thread::spawn(move || iterate_thread(&path_tx, bfs, &args_cloned, &halt_cloned));

    // Process all files in the queue
    let (mut file_errors, mut write_errors, mut is_aborted) = (u64::MIN, false, false);
    while let Ok(path_result) = path_rx.recv() {
        check_cancelled!(halt, is_aborted);
        let digest_result = match path_result {
            Ok(path) => match compute_file_digest(path, digest_size, args, halt) {
                Ok(digest_result) => digest_result,
                Err(Aborted) => {
                    is_aborted |= true;
                    break;
                }
            },
            Err(error) => Err(error),
        };
        if digest_result.is_err() {
            file_errors = file_errors.saturating_add(1u64);
        }
        if !print_result(output, &digest_result, digest_size, args) {
            write_errors = true;
            break;
        } else if !(digest_result.is_ok() || args.keep_going) {
            break;
        }
    }

    // Wait until iterating thread has completed
    drop(path_rx);
    if matches!(thread_handle.join().expect("Failed to join worker thread!"), Err(ThreadError::Aborted)) {
        is_aborted = true;
    }

    // Has the process been aborted?
    if is_aborted {
        return Err(Aborted);
    }

    // Print warning if any file(s) have been skipped
    if args.keep_going && (file_errors > 0u64) {
        print_error!(args, "WARNING: {} file(s) were skipped due to errors.", file_errors);
    }

    // Check for errors
    Ok((file_errors == u64::MIN) && (!write_errors))
}

// ---------------------------------------------------------------------------
// Process files
// ---------------------------------------------------------------------------

/// Process all input files
pub fn process_files(output: &mut impl Write, digest_size: usize, args: Arc<Args>, halt: Arc<Flag>) -> Result<bool, Aborted> {
    assert!(!args.files.is_empty(), "The list of input files must not be empty!");

    // Determine number of threads
    let thread_count = if args.multi_threading {
        match get_thread_count() {
            Ok(value) if value == usize::MIN => hardware_concurrency(),
            Ok(value) => NonZeroUsize::new(value).unwrap(),
            Err(error) => {
                print_error!(args, "Error: Invalid thread count \"{}\" specified!", error);
                return Ok(false);
            }
        }
    } else {
        NonZeroUsize::MIN
    };

    // Determine directory walking strategy
    let breadth_first = match get_search_strategy() {
        Ok(value) => value,
        Err(error) => {
            print_error!(args, "Error: Invalid directory walking strategy \"{}\" specified!", error);
            return Ok(false);
        }
    };

    if thread_count > NonZeroUsize::MIN {
        process_mt(output, thread_count.get(), digest_size, breadth_first, &args, &halt)
    } else {
        process_st(output, digest_size, breadth_first, &args, &halt)
    }
}

// ---------------------------------------------------------------------------
// Process STDIN
// ---------------------------------------------------------------------------

/// Process data from 'stdin' stream
pub fn process_stdin(output: &mut impl Write, digest_size: usize, args: Arc<Args>, halt: Arc<Flag>) -> Result<bool, Aborted> {
    let mut source = match DataSource::from_stdin() {
        Ok(stream) => stream,
        Err(_) => {
            print_error!(args, "Failed to acquire the standard input stream!");
            return Ok(false);
        }
    };

    let mut digest = [0u8; MAX_DIGEST_SIZE];

    match compute_digest(&mut source, &mut digest[..digest_size], &args, &halt) {
        Ok(_) => Ok(print_digest(output, &STDIN_NAME, &digest, digest_size, &args).is_ok()),
        Err(DigestError::IoError) => {
            print_error!(args, "Failed to read data from standard input stream!");
            Ok(false)
        }
        Err(DigestError::Aborted) => Err(Aborted),
    }
}
