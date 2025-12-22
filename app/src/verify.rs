// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::{bounded, Receiver, Sender};
use hex::decode_to_slice;
use num::Integer;
use std::{
    ffi::OsStr,
    io::{BufRead, BufReader, Read, Result as IoResult, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};
use tinyvec::TinyVec;

use crate::{
    arguments::Args,
    common::{get_capacity, increment, Aborted, Digest, Flag, TinyVecEx, MAX_DIGEST_SIZE},
    digest::{compute_digest, digest_equal, Error as DigestError},
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
    ChksumNotFound(PathBuf),
    ChksumObjIsDir(PathBuf),
    ChksumFileOpen(PathBuf),
    ChksumFileRead(PathBuf),
    ChksumStdnOpen,
    ChksumParseErr(PathBuf, usize),
    TargetNotFound(PathBuf),
    TargetObjIsDir(PathBuf),
    TargetFileOpen(PathBuf),
    TargetFileRead(PathBuf),
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

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

// Verification result
static VERIFICATION: [&str; 2usize] = ["FAILED", "OK"];

/// Print a single verification result
#[inline]
fn print_match(output: &mut impl Write, is_match: bool, file_name: &Path, args: &Args) -> IoResult<()> {
    if args.null {
        write!(output, "{}: {}\0", file_name.to_string_lossy(), VERIFICATION[is_match as usize])?;
    } else {
        writeln!(output, "{}: {}", file_name.to_string_lossy(), VERIFICATION[is_match as usize])?;
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
                Error::ChksumNotFound(path) => print_error!(args, "Checksum file not found: {:?}", path),
                Error::ChksumFileOpen(path) => print_error!(args, "Failed to open checksum file: {:?}", path),
                Error::ChksumFileRead(path) => print_error!(args, "Failed to read checksum file: {:?}", path),
                Error::ChksumParseErr(path, line) => print_error!(args, "Malformed checksum file: {:?} [line #{}]", path, line),
                Error::ChksumObjIsDir(path) => print_error!(args, "Checksum file is a directory: {:?}", path),
                Error::ChksumStdnOpen => print_error!(args, "Failed to acquire the standard input stream for reading!"),
                Error::TargetNotFound(path) => print_error!(args, "Target file not found: {:?}", path),
                Error::TargetFileOpen(path) => print_error!(args, "Failed to open target file: {:?}", path),
                Error::TargetFileRead(path) => print_error!(args, "Failed to read target file: {:?}", path),
                Error::TargetObjIsDir(path) => print_error!(args, "Target file is a directory: {:?}", path),
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

type VerifyResult = Result<(bool, PathBuf), Error>;

/// Compute checksum and compare to expected value
fn verify_checksum(source: &mut dyn Read, digest_expected: &[u8], args: &Args, halt: &Flag) -> Result<bool, DigestError> {
    let mut digest_computed: Digest = TinyVec::with_length(digest_expected.len());
    compute_digest(source, digest_computed.as_mut_slice(), args, halt)?;
    Ok(digest_equal(digest_computed.as_slice(), digest_expected))
}

/// Verify checksum of a single file
fn verify_file(file_name: PathBuf, digest_expected: &Digest, args: &Args, halt: &Flag) -> Result<VerifyResult, Cancelled> {
    match DataSource::from_path(&file_name) {
        Ok(mut file) => match verify_checksum(&mut file, digest_expected.as_slice(), args, halt) {
            Ok(is_match) => Ok(Ok((is_match, file_name))),
            Err(DigestError::IoError) => Ok(Err(Error::TargetFileRead(file_name))),
            Err(DigestError::Cancelled) => Err(Cancelled),
        },
        Err(error) => match error {
            IoError::FileNotFound => Ok(Err(Error::TargetNotFound(file_name))),
            IoError::IsADirectory => Ok(Err(Error::TargetObjIsDir(file_name))),
            _ => Ok(Err(Error::TargetFileOpen(file_name))),
        },
    }
}

/// Verify all provided checksums
fn verify_thread(checksum_rx: &Receiver<ReadResult>, result_tx: &Sender<VerifyResult>, args: &Args, halt: &Flag) -> TaskResult {
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

type ReadResult = Result<(Digest, PathBuf), Error>;
struct Malformed;

/// Parse a single line from checksum file
#[allow(clippy::collapsible_if)]
fn parse_checksum_line(line: &str) -> Result<(&OsStr, Digest), Malformed> {
    if let Some((digest_hex, input_name)) = line.split_once(|c: char| char::is_ascii_whitespace(&c)) {
        if (!digest_hex.is_empty()) && (!input_name.is_empty()) {
            let (length, remainder) = digest_hex.len().div_rem(&2usize);
            if (length > usize::MIN) && (length <= MAX_DIGEST_SIZE) && (remainder == usize::MIN) {
                let mut digest = TinyVec::with_length(length);
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
                        checksum_tx.send(Err(Error::ChksumParseErr(input_name.clone(), line_no + 1usize)))?;
                        if !args.keep_going {
                            return Ok(false);
                        }
                    }
                };
            }
            Err(_) => {
                checksum_tx.send(Err(Error::ChksumFileRead(input_name)))?;
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Read checksums from a file
fn read_checksum_file(checksum_tx: &Sender<ReadResult>, file_name: PathBuf, args: &Args, halt: &Flag) -> Result<bool, Cancelled> {
    match DataSource::from_path(&file_name) {
        Ok(mut file) => read_checksum_data(checksum_tx, &mut file, file_name, args, halt),
        Err(error) => {
            match error {
                IoError::FileNotFound => checksum_tx.send(Err(Error::ChksumNotFound(file_name)))?,
                IoError::IsADirectory => checksum_tx.send(Err(Error::ChksumObjIsDir(file_name)))?,
                _ => checksum_tx.send(Err(Error::ChksumFileOpen(file_name)))?,
            };
            Ok(false)
        }
    }
}

/// Iterate a list of checksum files
fn reader_thread(checksum_tx: &Sender<ReadResult>, args: &Args, halt: &Flag) -> TaskResult {
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
            Err(_) => checksum_tx.send(Err(Error::ChksumStdnOpen))?,
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Verify implementation
// ---------------------------------------------------------------------------

fn verify_mt(output: &mut impl Write, thread_count: NonZeroUsize, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Initialize channels
    let (checksum_tx, checksum_rx) = bounded::<ReadResult>(256usize);
    let (result_tx, result_rx) = bounded::<VerifyResult>(get_capacity(&thread_count));

    // Start the checksum reader thread
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    let thread_handle = thread::spawn(move || reader_thread(&checksum_tx, &args_cloned, &halt_cloned));

    // Start the worker threads
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    let thread_pool = ThreadPool::new(thread_count, move || verify_thread(&checksum_rx, &result_tx, &args_cloned, &halt_cloned));

    // Initialize counters
    let (mut chck_errors, mut file_errors, mut write_errors) = (u64::MIN, u64::MIN, false);

    // Process all verification results
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

    // Wait until the thread has completed
    if let Err(error) = thread_handle.join() {
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

    // Print warning if any file(s) did not match the expected checksum
    print_summary(chck_errors, file_errors, args);

    // Check for errors
    Ok((chck_errors == u64::MIN) && (file_errors == u64::MIN) && (!write_errors))
}

fn verify_st(output: &mut impl Write, args: &Arc<Args>, halt: &Arc<Flag>) -> Result<bool, Aborted> {
    // Initialize channel
    let (checksum_tx, checksum_rx) = bounded::<ReadResult>(256usize);

    // Start the checksum reader thread
    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    let thread_handle = thread::spawn(move || reader_thread(&checksum_tx, &args_cloned, &halt_cloned));

    // Initialize counters
    let (mut chck_errors, mut file_errors, mut write_errors) = (u64::MIN, u64::MIN, false);

    // Process all verification results
    while let Ok(checksum_result) = checksum_rx.recv() {
        break_cancelled!(halt);
        let verify_result = match checksum_result {
            Ok((digest_expected, file_name)) => match verify_file(file_name, &digest_expected, args, halt) {
                Ok(result) => result,
                Err(Cancelled) => break, /* cancelled */
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

    // Wait until the thread has completed
    if let Err(error) = thread_handle.join() {
        panic!("Failed to join the worker thread: {error:?}")
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

// ---------------------------------------------------------------------------
// Verify files
// ---------------------------------------------------------------------------

/// Verify all input files
pub fn verify_files(output: &mut impl Write, args: Arc<Args>, env: &Env, halt: Arc<Flag>) -> Result<bool, Aborted> {
    // Determine number of threads
    let thread_count = detect_thread_count(&args, env);

    if thread_count > NonZeroUsize::MIN {
        verify_mt(output, thread_count, &args, &halt)
    } else {
        verify_st(output, &args, &halt)
    }
}
