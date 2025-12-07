// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::{bounded, Receiver, SendError, Sender};
use hex::decode_to_slice;
use num::Integer;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader, Read, Result as IoResult, Write},
    iter,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};
use tinyvec::TinyVec;

use crate::{
    arguments::Args,
    common::{hardware_concurrency, increment, Aborted, Digest, Flag, TinyVecEx},
    digest::{compute_digest, digest_equal, Error as DigestError},
    environment::get_thread_count,
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
    ChksumSrcIsDir(PathBuf),
    ChksumFileOpen(PathBuf),
    ChksumFileRead(PathBuf),
    ChksumStdnOpen,
    ChksumParseErr(PathBuf, usize),
    TargetSrcIsDir(PathBuf),
    TargetFileOpen(PathBuf),
    TargetFileRead(PathBuf),
}

/// Error type to signal that a thread was cancelled
struct Cancelled;

impl<T> From<SendError<T>> for Cancelled {
    fn from(_: SendError<T>) -> Self {
        Self
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Check if the computation has been cancelled
macro_rules! check_cancelled {
    ($halt:ident) => {
        if $halt.cancelled() {
            return Err(Cancelled);
        }
    };
}

/// Check if the computation has been cancelled
macro_rules! break_cancelled {
    ($halt:ident) => {
        if $halt.cancelled() {
            break;
        }
    };
}

// ---------------------------------------------------------------------------
// Print results
// ---------------------------------------------------------------------------

// Verification result
static VERIFICATION_STATUS: [&str; 2usize] = ["FAILED", "OK"];

/// Print a single verification result
#[inline]
fn print_match(output: &mut impl Write, is_match: bool, file_name: &Path, args: &Args) -> IoResult<()> {
    if args.null {
        if args.plain {
            write!(output, "{}\0", VERIFICATION_STATUS[is_match as usize])?;
        } else {
            write!(output, "{}: {}\0", file_name.to_string_lossy(), VERIFICATION_STATUS[is_match as usize])?;
        }
    } else if args.plain {
        writeln!(output, "{}", VERIFICATION_STATUS[is_match as usize])?;
    } else {
        writeln!(output, "{}: {}", file_name.to_string_lossy(), VERIFICATION_STATUS[is_match as usize])?;
    }

    if args.flush {
        output.flush()?;
    }

    Ok(())
}

/// Print result to output
#[inline]
fn print_result(output: &mut impl Write, verify_result: &VerifyResult, args: &Args) -> bool {
    match verify_result {
        Ok((is_match, path)) => print_match(output, *is_match, path, args).is_ok(),
        Err(error) => {
            match error {
                TaskError::ChksumFileOpen(path) => print_error!(args, "Failed to open checksum file {:?}", path),
                TaskError::ChksumFileRead(path) => print_error!(args, "Failed to read checksum file {:?}", path),
                TaskError::ChksumParseErr(path, line) => print_error!(args, "Malformed checksum file {:?} [line #{}]", path, line),
                TaskError::ChksumSrcIsDir(path) => print_error!(args, "Checksum file is a directory: {:?}", path),
                TaskError::ChksumStdnOpen => print_error!(args, "Failed to acquire the standard input stream for reading!"),
                TaskError::TargetFileOpen(path) => print_error!(args, "Failed to open target file {:?}", path),
                TaskError::TargetFileRead(path) => print_error!(args, "Failed to read target file {:?}", path),
                TaskError::TargetSrcIsDir(path) => print_error!(args, "Target file is a directory: {:?}", path),
            }
            true
        }
    }
}

/// Print the summary
fn print_summary(chck_errors: u64, file_errors: u64, args: &Args) {
    if (chck_errors > u64::MIN) || (file_errors > u64::MIN) {
        if args.keep_going {
            if chck_errors > u64::MIN {
                print_error!(args, "WARNING: {} computed checksum(s) did *not* match!", chck_errors);
            }
            if file_errors > u64::MIN {
                print_error!(args, "WARNING: {} file(s) could not be verified due to errors!", file_errors);
            }
        } else {
            print_error!(args, "WARNING: The verification failed with an error!");
        }
    }
}

// ---------------------------------------------------------------------------
// Verify file digest
// ---------------------------------------------------------------------------

type VerifyResult = Result<(bool, PathBuf), TaskError>;

/// Compute checksum and compare to expected value
fn verify_checksum(source: &mut dyn Read, digest_expected: &[u8], args: &Args, halt: &Flag) -> Result<bool, DigestError> {
    let mut digest_computed: Digest = TinyVec::with_size(digest_expected.len());
    compute_digest(source, digest_computed.as_mut_slice(), args, halt)?;
    Ok(digest_equal(digest_computed.as_slice(), digest_expected))
}

/// Verify checksum of a single file
fn verify_file(file_name: PathBuf, digest_expected: &Digest, args: &Args, halt: &Flag) -> Result<VerifyResult, Cancelled> {
    match DataSource::from_path(&file_name) {
        Ok(mut file) => {
            if file.is_directory() {
                Ok(Err(TaskError::TargetSrcIsDir(file_name)))
            } else {
                match verify_checksum(&mut file, digest_expected.as_slice(), args, halt) {
                    Ok(is_match) => Ok(Ok((is_match, file_name))),
                    Err(DigestError::IoError) => Ok(Err(TaskError::TargetFileRead(file_name))),
                    Err(DigestError::Cancelled) => Err(Cancelled),
                }
            }
        }
        Err(_) => Ok(Err(TaskError::TargetFileOpen(file_name))),
    }
}

/// Verify all provided checksums
fn verify_thread(checksum_rx: &Receiver<ReadResult>, result_tx: &Sender<VerifyResult>, args: &Args, halt: &Flag) -> Result<(), Cancelled> {
    while let Ok(read_result) = checksum_rx.recv() {
        check_cancelled!(halt);
        match read_result {
            Ok((digest_expected, file_name)) => {
                let digest_result = verify_file(file_name, &digest_expected, args, halt)?;
                let is_success = matches!(digest_result, Ok((true, _)));
                result_tx.send(digest_result)?;
                if !(is_success || args.keep_going) {
                    break;
                }
            }
            Err(error) => result_tx.send(Err(error))?,
        }
    }

    Ok(())
}

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
                let mut digest = TinyVec::with_size(length);
                if decode_to_slice(digest_hex, digest.as_mut_slice()).is_ok() {
                    return Ok((OsStr::new(input_name), digest));
                }
            }
        }
    }

    Err(Malformed)
}

/// Read all checksums from source
fn read_checksum_data(checksum_tx: &Sender<ReadResult>, input: &mut dyn Read, input_name: PathBuf, args: &Args, halt: &Flag) -> Result<bool, Cancelled> {
    for (line_no, line) in BufReader::new(input).lines().enumerate() {
        check_cancelled!(halt);
        match line {
            Ok(line) => {
                let line_trimmed = line.trim_start();
                if !line_trimmed.is_empty() {
                    if let Ok((file_name, digest)) = parse_checksum_line(line_trimmed) {
                        checksum_tx.send(Ok((digest, PathBuf::from(file_name))))?;
                    } else {
                        checksum_tx.send(Err(TaskError::ChksumParseErr(input_name.clone(), line_no + 1usize)))?;
                        if !args.keep_going {
                            return Ok(false);
                        }
                    }
                };
            }
            Err(_) => {
                checksum_tx.send(Err(TaskError::ChksumFileRead(input_name)))?;
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Read checksums from a file
fn read_checksum_file(checksum_tx: &Sender<ReadResult>, file_name: PathBuf, args: &Args, halt: &Flag) -> Result<bool, Cancelled> {
    match File::open(&file_name) {
        Ok(mut file) => {
            if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                checksum_tx.send(Err(TaskError::ChksumSrcIsDir(file_name)))?;
                Ok(false)
            } else {
                read_checksum_data(checksum_tx, &mut file, file_name, args, halt)
            }
        }
        Err(_) => {
            checksum_tx.send(Err(TaskError::ChksumFileOpen(file_name)))?;
            Ok(false)
        }
    }
}

/// Iterate a list of checksum files
fn reader_thread(checksum_tx: &Sender<ReadResult>, args: &Args, halt: &Flag) -> Result<(), Cancelled> {
    if !args.files.is_empty() {
        for file_name in args.files.iter().cloned() {
            check_cancelled!(halt);
            if !(read_checksum_file(checksum_tx, file_name, args, halt)? || args.keep_going) {
                break;
            }
        }
    } else {
        match DataSource::from_stdin() {
            Ok(mut stdin_stream) => {
                read_checksum_data(checksum_tx, &mut stdin_stream, PathBuf::from(&*STDIN_NAME), args, halt)?;
            }
            Err(_) => checksum_tx.send(Err(TaskError::ChksumStdnOpen))?,
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
    let (result_tx, result_rx) = bounded::<VerifyResult>(thread_count);

    // Start the file iteration thread
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    thread_pool.push(thread::spawn(move || reader_thread(&checksum_tx, &args_cloned, &halt_cloned)));

    // Start the worker threads
    for (checksum_rx, result_tx) in iter::repeat_n(checksum_rx, thread_count).zip(iter::repeat_n(result_tx, thread_count)) {
        let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
        thread_pool.push(thread::spawn(move || verify_thread(&checksum_rx, &result_tx, &args_cloned, &halt_cloned)));
    }

    // Process all verification results
    let (mut chck_errors, mut file_errors, mut write_errors) = (u64::MIN, u64::MIN, false);
    while let Ok(verify_result) = result_rx.recv() {
        break_cancelled!(halt);
        let is_success = matches!(verify_result, Ok((true, _)));
        if verify_result.is_err() {
            increment(&mut file_errors)
        } else if !is_success {
            increment(&mut chck_errors)
        }

        if !print_result(output, &verify_result, args) {
            write_errors = true;
            break;
        } else if !(is_success || args.keep_going) {
            break;
        }
    }

    // Send shutdown signal to still running threads
    drop(result_rx);
    let is_aborted = halt.stop_process().is_err();

    // Wait until all threads have completed
    for thread in thread_pool.drain(..) {
        let _ = thread.join().expect("Failed to join worker thread!");
    }

    // Has the process been aborted?
    if is_aborted {
        return Err(Aborted);
    }

    // Print warning if any file(s) did not match the expected checksum
    print_summary(chck_errors, file_errors, args);

    // Check for errors
    Ok((chck_errors == u64::MIN) && (file_errors == u64::MIN) && (!write_errors))
}

fn verify_st(output: &mut impl Write, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Initialize channel
    let (checksum_tx, checksum_rx) = bounded::<ReadResult>(32usize);

    // Start the file iteration thread
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    let thread_handle = thread::spawn(move || reader_thread(&checksum_tx, &args_cloned, &halt_cloned));

    // Process all verification results
    let (mut chck_errors, mut file_errors, mut write_errors) = (u64::MIN, u64::MIN, false);
    while let Ok(checksum_result) = checksum_rx.recv() {
        break_cancelled!(halt);
        let verify_result = match checksum_result {
            Ok((digest_expected, file_name)) => match verify_file(file_name, &digest_expected, args, halt) {
                Ok(digest_result) => digest_result,
                Err(_) => break,
            },
            Err(error) => Err(error),
        };

        let is_success = matches!(verify_result, Ok((true, _)));
        if verify_result.is_err() {
            increment(&mut file_errors)
        } else if !is_success {
            increment(&mut chck_errors)
        }

        if !print_result(output, &verify_result, args) {
            write_errors = true;
            break;
        } else if !(is_success || args.keep_going) {
            break;
        }
    }

    // Send shutdown signal to still running threads
    drop(checksum_rx);
    let is_aborted = halt.stop_process().is_err();

    // Wait until the background thread has completed
    let _ = thread_handle.join().expect("Failed to join worker thread!");

    // Has the process been aborted?
    if is_aborted {
        return Err(Aborted);
    }

    // Print warning if any file(s) did not match the expected checksum
    print_summary(chck_errors, file_errors, args);

    // Check for errors
    Ok((chck_errors == u64::MIN) && (file_errors == u64::MIN) && (!write_errors))
}

// ---------------------------------------------------------------------------
// Verify files
// ---------------------------------------------------------------------------

/// Verify all input files
pub fn verify_files(output: &mut impl Write, args: Arc<Args>, halt: Arc<Flag>) -> Result<bool, Aborted> {
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
