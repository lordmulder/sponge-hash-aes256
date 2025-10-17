// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::decode_to_slice;
use num::Integer;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader, Read, Write, stdin},
    path::{Path, PathBuf},
    slice::Iter,
};

use crate::{
    arguments::Args,
    check_running,
    common::{Error, Flag, MAX_DIGEST_SIZE},
    digest::{compute_digest, digest_equal},
    handle_error, print_error,
};

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

fn print_summary(errors: &usize, faults: &usize, args: &Args) -> bool {
    if args.keep_going {
        if *faults > 0usize {
            print_error!(args, "Warning: {} computed checksum(s) did *not* match.", *faults);
        }
        if *errors > 0usize {
            print_error!(args, "Warning: {} additional error(s) were encountered.", *errors);
        }
    }

    (*errors == 0usize) && (*faults == 0usize)
}

// ---------------------------------------------------------------------------
// Verify checksum
// ---------------------------------------------------------------------------

static RESULT_TEXT: [&str; 2usize] = ["FAILED", "OK"];

/// Compute checksum and compare to expected value
fn verify_checksum(input: &mut impl Read, digest_expected: &[u8], output: &mut impl Write, name: &OsStr, args: &Args, running: &Flag) -> Result<bool, Error> {
    let digest_size = digest_expected.len();
    let mut digest_computed = [0u8; MAX_DIGEST_SIZE];

    compute_digest(input, &mut digest_computed[..digest_size], args, running)?;
    let is_match = digest_equal(&digest_computed[..digest_size], &digest_expected[..digest_size]);

    if args.null {
        if args.plain {
            write!(output, "{}\0", RESULT_TEXT[is_match as usize])?;
        } else {
            write!(output, "{}: {}\0", name.to_string_lossy(), RESULT_TEXT[is_match as usize])?;
        }
    } else if args.plain {
        writeln!(output, "{}", RESULT_TEXT[is_match as usize])?;
    } else {
        writeln!(output, "{}: {}", name.to_string_lossy(), RESULT_TEXT[is_match as usize])?;
    }

    if args.flush {
        output.flush()?;
    }

    Ok(is_match)
}

/// Verify checksum of a single file
fn verify_file(path: &Path, digest_expected: &[u8], output: &mut impl Write, args: &Args, running: &Flag, errors: &mut usize, faults: &mut usize) -> bool {
    match File::open(path) {
        Ok(mut file) => {
            if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                handle_error!(args, errors, "Input file is a directory: {:?}", path);
            } else {
                match verify_checksum(&mut file, digest_expected, output, path.as_os_str(), args, running) {
                    Ok(true) => {}
                    Ok(false) => {
                        if args.keep_going {
                            *faults += 1usize;
                        } else {
                            return false;
                        }
                    }
                    Err(Error::Aborted) => {
                        print_error!(args, "Aborted: The process has been interrupted by the user!");
                        return false;
                    }
                    Err(error) => handle_error!(args, errors, "Failed to verify file: {:?} [{:?}]", path, error),
                }
            }
        }
        Err(error) => handle_error!(args, errors, "Failed to open input file: {:?} [{:?}]", path, error),
    }

    true
}

// ---------------------------------------------------------------------------
// Process checksum file
// ---------------------------------------------------------------------------

/// Parse a line from checksum file
fn parse_line<'a, 'b>(line: &'a str, digest: &'b mut [u8; MAX_DIGEST_SIZE]) -> Option<(&'a Path, &'b [u8])> {
    if let Some((digest_hex, file_name)) = line.split_once(|c: char| char::is_ascii_whitespace(&c)) {
        let (length, remainder) = digest_hex.len().div_rem(&2usize);
        if (length > 0usize) && (remainder == 0usize) && (!file_name.is_empty()) && decode_to_slice(digest_hex, &mut digest[..length]).is_ok() {
            return Some((Path::new(file_name), &digest[..length]));
        }
    }

    None
}

/// Process a single input file
fn verify_checksums(input: &mut impl Read, output: &mut impl Write, args: &Args, running: &Flag, errors: &mut usize, faults: &mut usize) -> bool {
    let mut digest_buffer = [0u8; MAX_DIGEST_SIZE];

    for (line_no, line) in BufReader::new(input).lines().enumerate() {
        check_running!(args, running);
        match line {
            Ok(line) => {
                let line_trimmed = line.trim_ascii_start();
                if !line_trimmed.is_empty() {
                    if let Some((file_name, digest_expected)) = parse_line(line_trimmed, &mut digest_buffer) {
                        if !verify_file(file_name, digest_expected, output, args, running, errors, faults) {
                            return false;
                        }
                    } else {
                        handle_error!(args, errors, "Error: Malformed checksum at line #{} encountered!", line_no + 1usize);
                    }
                };
            }
            Err(error) => handle_error!(args, errors, "Failed to read checksum from file: {:?}", error),
        }
    }

    true
}

// ---------------------------------------------------------------------------
// Read checksums
// ---------------------------------------------------------------------------

/// Read checksums from a file
fn read_checksum_file(path: &PathBuf, output: &mut impl Write, args: &Args, running: &Flag, errors: &mut usize, faults: &mut usize) -> bool {
    match File::open(path) {
        Ok(mut file) => {
            if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                handle_error!(args, errors, "Checksum file is a directory: {:?}", path);
            } else {
                return verify_checksums(&mut file, output, args, running, errors, faults);
            }
        }
        Err(error) => handle_error!(args, errors, "Failed to open checksum file: {:?} [{:?}]", path, error),
    }

    true
}

pub fn verify_from_stdin(output: &mut impl Write, args: &Args, running: Flag) -> bool {
    let mut input = stdin().lock();
    let (mut errors, mut faults) = (0usize, 0usize);

    if !verify_checksums(&mut input, output, args, &running, &mut errors, &mut faults) {
        return false;
    }

    print_summary(&errors, &faults, args)
}

// ---------------------------------------------------------------------------
// Iterate checksum files
// ---------------------------------------------------------------------------

/// Iterate a list of checksum files
pub fn verify_files(files: Iter<'_, PathBuf>, output: &mut impl Write, args: &Args, running: Flag) -> bool {
    let (mut errors, mut faults) = (0usize, 0usize);

    for file_name in files {
        check_running!(args, running);
        if !read_checksum_file(file_name, output, args, &running, &mut errors, &mut faults) {
            return false;
        }
    }

    print_summary(&errors, &faults, args)
}
