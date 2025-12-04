// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::{bounded, SendError, Sender};
use hex::decode_to_slice;
use num::Integer;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    num::NonZeroUsize,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
    thread,
};

use crate::{
    arguments::Args,
    common::{calloc_vec, hardware_concurrency, Aborted, Digest, Flag},
    environment::get_thread_count,
    print_error,
};

// ---------------------------------------------------------------------------
// Error Type
// ---------------------------------------------------------------------------

/// Error type for processing file tasks
#[derive(Debug)]
#[allow(dead_code)]
enum TaskError {
    CheckSrcIsDir(PathBuf),
    CheckFileOpen(PathBuf),
    CheckFileRead(PathBuf),
    CheckParseErr(PathBuf, usize),
    Undefined,
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
// Utility functions
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Compute file digest
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Read checksums from checksum file
// ---------------------------------------------------------------------------

type ReadResult = Result<(Digest, PathBuf), TaskError>;
struct Malformed;

/// Parse a single line from checksum file
#[allow(clippy::collapsible_if)]
fn parse_checksum_line(line: &str) -> Result<(&OsStr, Digest), Malformed> {
    if let Some((digest_hex, input_name)) = line.split_once(|c: char| char::is_ascii_whitespace(&c)) {
        if (!digest_hex.is_empty()) && (!input_name.is_empty()) {
            let (length, remainder) = digest_hex.len().div_rem(&2usize);
            if (length > usize::MIN) && (remainder == usize::MIN) {
                let mut digest = calloc_vec(length);
                if decode_to_slice(digest_hex, digest.as_mut_slice()).is_ok() {
                    return Ok((OsStr::new(input_name), digest));
                }
            }
        }
    }

    Err(Malformed)
}

/// Read all checksums from source
fn read_checksum_data(checksum_tx: &Sender<ReadResult>, input: &mut dyn Read, input_name: PathBuf, args: &Args, halt: &Flag) -> Result<bool, ThreadError> {
    for (line_no, line) in BufReader::new(input).lines().enumerate() {
        check_cancelled!(halt);
        match line {
            Ok(line) => {
                let line_trimmed = line.trim_start();
                if !line_trimmed.is_empty() {
                    if let Ok((file_name, digest)) = parse_checksum_line(line_trimmed) {
                        checksum_tx.send(Ok((digest, PathBuf::from(file_name))))?;
                    } else {
                        checksum_tx.send(Err(TaskError::CheckParseErr(input_name.clone(), line_no + 1usize)))?;
                        if !args.keep_going {
                            return Ok(false);
                        }
                    }
                };
            }
            Err(_) => {
                checksum_tx.send(Err(TaskError::CheckFileRead(input_name)))?;
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Read checksums from a file
fn read_checksum_file(checksum_tx: &Sender<ReadResult>, file_name: PathBuf, args: &Args, halt: &Flag) -> Result<bool, ThreadError> {
    match File::open(&file_name) {
        Ok(mut file) => {
            if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                checksum_tx.send(Err(TaskError::CheckSrcIsDir(file_name)))?;
                Ok(false)
            } else {
                read_checksum_data(checksum_tx, &mut file, file_name, args, halt)
            }
        }
        Err(_) => {
            checksum_tx.send(Err(TaskError::CheckFileOpen(file_name)))?;
            Ok(false)
        }
    }
}

/// Iterate a list of checksum files
fn reader_thread(checksum_tx: &Sender<ReadResult>, args: &Args, halt: &Flag) -> Result<(), ThreadError> {
    for file_name in args.files.iter().cloned() {
        check_cancelled!(halt);
        if !(read_checksum_file(checksum_tx, file_name, args, halt)? || args.keep_going) {
            break;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Verify implementation
// ---------------------------------------------------------------------------

fn verify_mt(output: &mut impl Write, thread_count: usize, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Initialize thread pool
    let mut thread_pool = Vec::with_capacity(thread_count.saturating_add(1usize));
    let (checksum_tx, checksum_rx) = bounded::<ReadResult>(thread_count.saturating_mul(16usize));
    //let (_digest_tx, digest_rx) = bounded::<()>(thread_count);

    // Start the file iteration thread
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    thread_pool.push(thread::spawn(move || reader_thread(&checksum_tx, &args_cloned, &halt_cloned)));

    // Start the worker threads
    /*for (path_rx, digest_tx) in iter::repeat_n(path_rx, thread_count).zip(iter::repeat_n(digest_tx, thread_count)) {
        let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
        //thread_pool.push(thread::spawn(move || compute_thread(&path_rx, &digest_tx, &args_cloned, &halt_cloned)));
    }*/

    // Process all digest results
    let (file_errors, mut write_errors, mut is_aborted) = (u64::MIN, false, false);
    /*while let Ok(digest_result) = digest_rx.recv() {
        check_cancelled!(halt, is_aborted);
        if digest_result.is_err() {
            file_errors = file_errors.saturating_add(1u64);
        }
        if !print_result(output, &digest_result, args) {
            write_errors = true;
            break;
        } else if !(digest_result.is_ok() || args.keep_going) {
            break;
        }
    }*/

    // TEST
    while let Ok(checksum) = checksum_rx.recv() {
        if writeln!(output, "Checksum: {:?}", checksum).is_err() {
            write_errors = true;
            break;
        }
    }

    // Wait until all threads have completed
    drop(checksum_rx);
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

fn verify_st(_output: &mut impl Write, _args: &Arc<Args>, _halt: &Arc<Flag>) -> Result<bool, Aborted> {
    todo!("verify_st()");
    /*
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
    Ok((file_errors == u64::MIN) && (!write_errors))  */
}

// ---------------------------------------------------------------------------
// Process files
// ---------------------------------------------------------------------------

/// Process all input files
pub fn verify_files(output: &mut impl Write, args: Arc<Args>, halt: Arc<Flag>) -> Result<bool, Aborted> {
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

    if thread_count > NonZeroUsize::MIN {
        verify_mt(output, thread_count.get(), &args, &halt)
    } else {
        verify_st(output, &args, &halt)
    }
}

// ---------------------------------------------------------------------------
// Verify from STDIN
// ---------------------------------------------------------------------------

/// Process data from 'stdin' stream
pub fn verify_stdin(_output: &mut impl Write, _args: Arc<Args>, _halt: Arc<Flag>) -> Result<bool, Aborted> {
    todo!("verify_stdin()");
    /*let mut source = match DataSource::from_stdin() {
        Ok(stream) => stream,
        Err(_) => {
            print_error!(args, "Failed to acquire the standard input stream!");
            return Ok(false);
        }
    };

    let mut digest = EMPTY_DIGEST;

    match compute_digest(&mut source, &mut digest[..digest_size], &args, &halt) {
        Ok(_) => Ok(print_digest(output, &STDIN_NAME, &digest, digest_size, &args).is_ok()),
        Err(DigestError::IoError) => {
            print_error!(args, "Failed to read data from standard input stream!");
            Ok(false)
        }
        Err(DigestError::Aborted) => Err(Aborted),
    }*/
}
