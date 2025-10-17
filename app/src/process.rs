// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use std::{
    ffi::OsStr,
    fs::{DirEntry, metadata, read_dir},
    io::{Read, Write},
    path::PathBuf,
    slice::Iter,
    str::from_utf8,
};

use crate::{
    arguments::Args,
    check_running,
    common::{Error, Flag, MAX_DIGEST_SIZE},
    digest::compute_digest,
    handle_error,
    io::{DataSource, STDIN_NAME},
    print_error,
};

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Check if directory entry is a directory or a symlink to a directory
fn is_directory(entry: &DirEntry) -> bool {
    match entry.metadata() {
        Ok(meta) => {
            let file_type = meta.file_type();
            match file_type.is_dir() {
                true => true,
                false => match file_type.is_symlink() {
                    true => metadata(entry.path()).is_ok_and(|info| info.is_dir()),
                    false => false,
                },
            }
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Process file
// ---------------------------------------------------------------------------

/// Process a single input file
fn process_file(input: &mut dyn Read, output: &mut impl Write, name: &OsStr, digest_size: usize, args: &Args, running: &Flag) -> Result<(), Error> {
    let mut digest = [0u8; MAX_DIGEST_SIZE];
    compute_digest(input, &mut digest[..digest_size], args, running)?;

    let mut hex_buffer = [0u8; MAX_DIGEST_SIZE * 2usize];
    let hex_str = &mut hex_buffer[..digest_size.checked_mul(2usize).unwrap()];
    encode_to_slice(&digest[..digest_size], hex_str).unwrap();

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
fn read_file(path: &PathBuf, output: &mut impl Write, digest_size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    match DataSource::from_path(path) {
        Ok(mut file) => {
            if file.is_directory() {
                handle_error!(args, errors, "Input file is a directory: {:?}", path);
            } else {
                match process_file(&mut file, output, path.as_os_str(), digest_size, args, running) {
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
pub fn process_from_stdin(output: &mut impl Write, digest_size: usize, args: &Args, running: Flag) -> bool {
    let mut input = match DataSource::from_stdin() {
        Ok(stream) => stream,
        Err(error) => {
            print_error!(args, "Failed to acquire the standard input stream: {}", error);
            return false;
        }
    };

    match process_file(&mut input, output, &STDIN_NAME, digest_size, args, &running) {
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
fn process_directory(path: &PathBuf, output: &mut impl Write, digest_size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    match read_dir(path) {
        Ok(dir_iter) => {
            for element in dir_iter {
                check_running!(args, running);
                match element {
                    Ok(dir_entry) => {
                        if is_directory(&dir_entry) {
                            if args.recursive && (!process_directory(&dir_entry.path(), output, digest_size, args, running, errors)) {
                                return false;
                            }
                        } else if !read_file(&dir_entry.path(), output, digest_size, args, running, errors) {
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

    true
}

/// Iterate a list of input files
pub fn process_files(files: Iter<'_, PathBuf>, output: &mut impl Write, digest_size: usize, args: &Args, running: Flag) -> bool {
    let mut errors = 0usize;
    let handle_dirs = args.dirs || args.recursive;

    for file_name in files {
        check_running!(args, running);
        if handle_dirs && metadata(file_name).is_ok_and(|meta| meta.is_dir()) {
            if !process_directory(file_name, output, digest_size, args, &running, &mut errors) {
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
