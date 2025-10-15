// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use sponge_hash_aes256::SpongeHash256;
use std::{
    ffi::OsStr,
    fs::{DirEntry, File, metadata, read_dir},
    io::{BufRead, BufReader, Read, Write, stdin},
    path::PathBuf,
    slice::Iter,
    str::from_utf8,
};

use crate::{
    arguments::Args,
    check_running,
    common::{Error, Flag, MAX_DIGEST_SIZE, MAX_SNAIL_LEVEL},
    handle_error, print_error,
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
// Hasher
// ---------------------------------------------------------------------------

const SNAIL_ITERATIONS_1: usize = 13usize;
const SNAIL_ITERATIONS_2: usize = 251usize;
const SNAIL_ITERATIONS_3: usize = 4093usize;
const SNAIL_ITERATIONS_4: usize = 65521usize;

enum Hasher {
    Default(SpongeHash256),
    SnailV1(SpongeHash256<SNAIL_ITERATIONS_1>),
    SnailV2(SpongeHash256<SNAIL_ITERATIONS_2>),
    SnailV3(SpongeHash256<SNAIL_ITERATIONS_3>),
    SnailV4(SpongeHash256<SNAIL_ITERATIONS_4>),
}

impl Hasher {
    #[inline(always)]
    pub fn new(info: &Option<String>, snail_level: u8) -> Self {
        assert!(snail_level <= MAX_SNAIL_LEVEL);
        match info {
            Some(info) => match snail_level {
                0u8 => Self::Default(SpongeHash256::with_info(info)),
                1u8 => Self::SnailV1(SpongeHash256::with_info(info)),
                2u8 => Self::SnailV2(SpongeHash256::with_info(info)),
                3u8 => Self::SnailV3(SpongeHash256::with_info(info)),
                4u8 => Self::SnailV4(SpongeHash256::with_info(info)),
                _ => unreachable!(),
            },
            None => match snail_level {
                0u8 => Self::Default(SpongeHash256::new()),
                1u8 => Self::SnailV1(SpongeHash256::new()),
                2u8 => Self::SnailV2(SpongeHash256::new()),
                3u8 => Self::SnailV3(SpongeHash256::new()),
                4u8 => Self::SnailV4(SpongeHash256::new()),
                _ => unreachable!(),
            },
        }
    }

    #[inline(always)]
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        match self {
            Hasher::Default(hasher) => hasher.update(input),
            Hasher::SnailV1(hasher) => hasher.update(input),
            Hasher::SnailV2(hasher) => hasher.update(input),
            Hasher::SnailV3(hasher) => hasher.update(input),
            Hasher::SnailV4(hasher) => hasher.update(input),
        }
    }

    #[inline(always)]
    pub fn digest_to_slice(self, output: &mut [u8]) {
        match self {
            Hasher::Default(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV1(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV2(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV3(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV4(hasher) => hasher.digest_to_slice(output),
        }
    }
}

// ---------------------------------------------------------------------------
// Print digest
// ---------------------------------------------------------------------------

fn print_digest(output: &mut impl Write, digest: &[u8], name: &OsStr, args: &Args) -> Result<(), Error> {
    assert!(digest.len() <= MAX_DIGEST_SIZE, "Length of digest exceeds allowable maximum!");

    let mut hex_buffer = [0u8; MAX_DIGEST_SIZE * 2usize];
    let hex_str = &mut hex_buffer[..digest.len().checked_mul(2usize).unwrap()];

    encode_to_slice(digest, hex_str).unwrap();

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
// Process file
// ---------------------------------------------------------------------------

#[cfg(target_pointer_width = "64")]
const IO_BUFFER_SIZE: usize = 8192usize;
#[cfg(target_pointer_width = "32")]
const IO_BUFFER_SIZE: usize = 4096usize;
#[cfg(target_pointer_width = "16")]
const IO_BUFFER_SIZE: usize = 2048usize;

/// Process a single input file
fn process_file(input: &mut impl Read, output: &mut impl Write, name: &OsStr, size: usize, args: &Args, running: &Flag) -> Result<(), Error> {
    let mut digest = [0u8; MAX_DIGEST_SIZE];
    let mut hasher = Hasher::new(&args.info, args.snail);

    if !args.text {
        let mut buffer = [0u8; IO_BUFFER_SIZE];
        loop {
            check_running!(running);
            match input.read(&mut buffer)? {
                0 => break,
                length => hasher.update(&buffer[..length]),
            }
        }
    } else {
        let mut lines = BufReader::new(input).lines();
        const LINE_BREAK: &str = "\n";
        if let Some(line) = lines.next() {
            hasher.update(&(line?));
            for line in lines {
                check_running!(running);
                hasher.update(LINE_BREAK);
                hasher.update(&(line?));
            }
        }
    }

    hasher.digest_to_slice(&mut digest[..size]);
    print_digest(output, &digest[..size], name, args)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Read input file or stream
// ---------------------------------------------------------------------------

/// Read data from a file
fn read_file(path: &PathBuf, output: &mut impl Write, digest_size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    match File::open(path) {
        Ok(mut file) => {
            if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                handle_error!(args, errors, "Input is a directory: {:?}", path);
            } else {
                match process_file(&mut file, output, path.as_os_str(), digest_size, args, running) {
                    Ok(_) => {}
                    Err(Error::Aborted) => {
                        print_error!(args, "Aborted: The process has been interrupted by the user!");
                        return false;
                    }
                    Err(error) => handle_error!(args, errors, "Failed to process file: {:?} [{:?}]", path, error),
                }
            }
        }
        Err(error) => handle_error!(args, errors, "Failed to open input file: {:?} [{:?}]", path, error),
    }

    true
}

/// Read data from the `stdin` stream
pub fn read_from_stdin(output: &mut impl Write, digest_size: usize, args: &Args, running: Flag) -> bool {
    let mut input = stdin().lock();

    match process_file(&mut input, output, OsStr::new("-"), digest_size, args, &running) {
        Ok(_) => true,
        Err(Error::Aborted) => {
            print_error!(args, "Aborted: The process has been interrupted by the user!");
            false
        }
        Err(error) => {
            print_error!(args, "Failed to process input data from 'stdin' stream: {:?}", error);
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Iterate input files/directories
// ---------------------------------------------------------------------------

/// Iterate all files and sub-directories in a directory
fn iterate_directory(path: &PathBuf, output: &mut impl Write, digest_size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    match read_dir(path) {
        Ok(dir_iter) => {
            for element in dir_iter {
                check_running!(args, running);
                match element {
                    Ok(dir_entry) => {
                        if is_directory(&dir_entry) {
                            if args.recursive && (!iterate_directory(&dir_entry.path(), output, digest_size, args, running, errors)) {
                                return false;
                            }
                        } else if !read_file(&dir_entry.path(), output, digest_size, args, running, errors) {
                            return false;
                        }
                    }
                    Err(error) => {
                        handle_error!(args, errors, "Failed to read directory: {:?} [{:?}]", path, error);
                    }
                }
            }
        }
        Err(error) => {
            handle_error!(args, errors, "Failed to open directory: {:?} [{:?}]", path, error);
        }
    }

    true
}

/// Iterate a list of input files
pub fn iterate_files(files: Iter<'_, PathBuf>, output: &mut impl Write, digest_size: usize, args: &Args, running: Flag) -> bool {
    let mut errors = 0usize;
    let handle_dirs = args.dirs || args.recursive;

    for file_name in files {
        check_running!(args, running);
        if handle_dirs && metadata(file_name).is_ok_and(|meta| meta.is_dir()) {
            if !iterate_directory(file_name, output, digest_size, args, &running, &mut errors) {
                return false;
            }
        } else if !read_file(file_name, output, digest_size, args, &running, &mut errors) {
            return false;
        }
    }

    if args.keep_going && (errors > 0usize) {
        print_error!(args, "Warning: {} file(s) were skipped due to errors.", errors);
    }

    errors == 0usize
}
