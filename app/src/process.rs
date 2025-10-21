// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use std::{
    ffi::OsStr,
    fs::{DirEntry, Metadata, metadata, read_dir},
    io::{Read, Write},
    path::PathBuf,
    slice::Iter,
    str::from_utf8,
};

#[cfg(unix)]
use std::{collections::BTreeSet, os::unix::fs::MetadataExt};

use crate::{
    arguments::Args,
    check_running,
    common::{Error, Flag, MAX_DIGEST_SIZE},
    digest::compute_digest,
    handle_error,
    io::{DataSource, STDIN_NAME},
    print_error,
};

/// Data type used to store visited directories on Unix
#[cfg(unix)]
type SetType = BTreeSet<u128>;

/// Dummy data type to be used on other platform
#[cfg(not(unix))]
type SetType = ();

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
                    true => metadata(dir_entry.path()).ok().filter(|value| value.is_dir()),
                    false => None,
                },
                true => Some(meta_data),
            }
        }
        Err(_) => None,
    }
}

/// Combine two separate u64 values to a single u128 value
#[cfg(unix)]
#[inline(always)]
fn make_u128(high: u64, low: u64) -> u128 {
    ((high as u128) << 64usize) | (low as u128)
}

/// Make sure that the directory was not visited yet
#[cfg(unix)]
#[inline(always)]
fn not_visited(meta: &Metadata, visited: &mut SetType) -> bool {
    visited.insert(make_u128(meta.dev(), meta.ino()))
}

/// On platforms other than Unix we simply return `true`
#[cfg(not(unix))]
#[inline(always)]
fn not_visited(_: &Metadata, _: &mut SetType) -> bool {
    true
}

// ---------------------------------------------------------------------------
// Process file
// ---------------------------------------------------------------------------

/// Process a single input file
fn process_file(input: &mut dyn Read, output: &mut impl Write, name: &OsStr, size: usize, args: &Args, running: &Flag) -> Result<(), Error> {
    let mut digest = [0u8; MAX_DIGEST_SIZE];
    compute_digest(input, &mut digest[..size], args, running)?;

    let mut hex_buffer = [0u8; MAX_DIGEST_SIZE * 2usize];
    let hex_str = &mut hex_buffer[..size.checked_mul(2usize).unwrap()];
    encode_to_slice(&digest[..size], hex_str).unwrap();

    if args.null {
        if args.plain {
            write!(output, "{}\0", from_utf8(hex_str).unwrap())?;
        } else {
            write!(output, "{} {}\0", from_utf8(hex_str).unwrap(), name.to_string_lossy())?;
        }
    } else if args.plain {
        writeln!(output, "{}", from_utf8(hex_str).unwrap())?;
    } else {
        writeln!(output, "{} {}", from_utf8(hex_str).unwrap(), name.to_string_lossy())?;
    }

    if args.flush {
        output.flush()?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Read input file or stream
// ---------------------------------------------------------------------------

/// Read data from a file
fn read_file(path: &PathBuf, output: &mut impl Write, size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    match DataSource::from_path(path) {
        Ok(mut file) => {
            if file.is_directory() {
                handle_error!(args, errors, "Input file is a directory: {:?}", path);
            } else {
                match process_file(&mut file, output, path.as_os_str(), size, args, running) {
                    Ok(_) => {}
                    Err(Error::Aborted) => {
                        print_error!(args, "Aborted: The process has been interrupted by the user!");
                        return false;
                    }
                    Err(error) => handle_error!(args, errors, "Failed to process file: {:?} ({})", path, error),
                }
            }
        }
        Err(error) => handle_error!(args, errors, "Failed to open input file: {:?} ({})", path, error),
    }

    true
}

/// Read data from the `stdin` stream
pub fn process_from_stdin(output: &mut impl Write, size: usize, args: &Args, running: Flag) -> bool {
    let mut input = match DataSource::from_stdin() {
        Ok(stream) => stream,
        Err(error) => {
            print_error!(args, "Failed to acquire the standard input stream: {}", error);
            return false;
        }
    };

    match process_file(&mut input, output, &STDIN_NAME, size, args, &running) {
        Ok(_) => true,
        Err(Error::Aborted) => {
            print_error!(args, "Aborted: The process has been interrupted by the user!");
            false
        }
        Err(error) => {
            print_error!(args, "Failed to process input data from 'stdin' stream: {}", error);
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Iterate input files/directories
// ---------------------------------------------------------------------------

/// Iterate all files and sub-directories in a directory
fn process_directory(path: &PathBuf, visited: &mut SetType, output: &mut impl Write, size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    let mut dir_queue = if args.recursive { Vec::with_capacity(64usize) } else { Vec::new() };

    match read_dir(path) {
        Ok(dir_iter) => {
            for element in dir_iter {
                check_running!(args, running);
                match element {
                    Ok(dir_entry) => {
                        if let Some(meta_data) = is_directory(&dir_entry) {
                            if args.recursive && not_visited(&meta_data, visited) {
                                dir_queue.push(dir_entry.path());
                            }
                        } else if !read_file(&dir_entry.path(), output, size, args, running, errors) {
                            return false;
                        }
                    }
                    Err(error) => {
                        handle_error!(args, errors, "Failed to read directory: {:?} ({})", path, error);
                    }
                }
            }
        }
        Err(error) => {
            handle_error!(args, errors, "Failed to open directory: {:?} ({})", path, error);
        }
    }

    for dir_name in dir_queue.into_iter() {
        if !process_directory(&dir_name, visited, output, size, args, running, errors) {
            return false;
        }
    }

    true
}

/// Iterate a list of input files
pub fn process_files(files: Iter<'_, PathBuf>, output: &mut impl Write, digest_size: usize, args: &Args, running: Flag) -> bool {
    let mut errors = 0usize;
    let handle_dirs = args.dirs || args.recursive;

    for file_name in files {
        check_running!(args, running);
        if handle_dirs && metadata(file_name).is_ok_and(|meta| meta.is_dir()) {
            if !process_directory(file_name, &mut SetType::default(), output, digest_size, args, &running, &mut errors) {
                return false;
            }
        } else if !read_file(file_name, output, digest_size, args, &running, &mut errors) {
            return false;
        }
    }

    if args.keep_going && (errors > 0usize) {
        print_error!(args, "WARNING: {} file(s) were skipped due to errors.", errors);
    }

    errors == 0usize
}
