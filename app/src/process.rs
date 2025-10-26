// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use std::{
    borrow::Cow,
    collections::BTreeSet,
    ffi::OsStr,
    fs::{self, DirEntry, Metadata},
    io::{Read, Write},
    iter,
    path::PathBuf,
    slice::Iter,
    str::from_utf8,
    sync::OnceLock,
};

use crate::{
    abort,
    arguments::Args,
    check_running,
    common::{get_env, parse_enum, Error, Flag, MAX_DIGEST_SIZE},
    digest::compute_digest,
    handle_error,
    io::{DataSource, STDIN_NAME},
    print_error,
};

/// Data type used to store already visited directories
type FileId = (u64, u64);
type SetType = BTreeSet<FileId>;

// ---------------------------------------------------------------------------
// Platform support
// ---------------------------------------------------------------------------

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
fn append(visited: &'_ SetType, file_id: Option<FileId>) -> Cow<'_, SetType> {
    file_id.map_or(Cow::Borrowed(visited), |id| {
        let mut cloned = visited.clone();
        assert!(cloned.insert(id), "Failed to insert file id!");
        Cow::Owned(cloned)
    })
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
                    Err(Error::Aborted) => abort!(args),
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
        Err(Error::Aborted) => abort!(args),
        Err(error) => {
            print_error!(args, "Failed to process input data from 'stdin' stream: {}", error);
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Iterate input files/directories
// ---------------------------------------------------------------------------

/// Enable breadth-first search?
fn parse_search_strategy(args: &Args) -> bool {
    static SEARCH_STRATEGY: OnceLock<Option<bool>> = OnceLock::new();
    SEARCH_STRATEGY
        .get_or_init(|| {
            get_env("SPONGE256SUM_DIRWALK_STRATEGY").and_then(|value| match parse_enum(value, &["BFS", "DFS"]) {
                Some(position) => Some(position == 0usize),
                None => {
                    print_error!(args, "Invalid directory search strategy: {:?}", value);
                    None
                }
            })
        })
        .unwrap_or(true)
}

/// Iterate all files and sub-directories in a directory
#[allow(clippy::unnecessary_map_or)]
fn process_directory(path: &PathBuf, visited: &SetType, output: &mut impl Write, size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    let mut dir_queue: Option<Vec<(Option<FileId>, PathBuf)>> = None;

    match fs::read_dir(path) {
        Ok(dir_iter) => {
            let breadth_first = if args.recursive { Some(parse_search_strategy(args)) } else { None };
            for element in dir_iter {
                check_running!(args, running);
                match element {
                    Ok(dir_entry) => {
                        if let Some(meta_data) = is_directory(&dir_entry) {
                            if args.recursive {
                                let file_id = file_id::get(&meta_data);
                                if file_id.map_or(true, |id| !visited.contains(&id)) {
                                    if breadth_first.unwrap() {
                                        dir_queue.get_or_insert_with(|| Vec::with_capacity(64usize)).push((file_id, dir_entry.path()));
                                    } else if !process_directory(&dir_entry.path(), &append(visited, file_id), output, size, args, running, errors) {
                                        return false;
                                    }
                                } else {
                                    print_error!(args, "File system loop detected, skipping: {:?}", dir_entry.path());
                                }
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

    if let Some(queue) = dir_queue {
        for (file_id, dir_name) in queue.into_iter() {
            if !process_directory(&dir_name, &append(visited, file_id), output, size, args, running, errors) {
                return false;
            }
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
        let dir_info = if handle_dirs { fs::metadata(file_name).ok().filter(|meta| meta.is_dir()) } else { None };
        if let Some(meta_data) = dir_info {
            let visited = file_id::get(&meta_data).map_or_else(SetType::new, |dir_id| iter::once(dir_id).collect());
            if !process_directory(file_name, &visited, output, digest_size, args, &running, &mut errors) {
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
