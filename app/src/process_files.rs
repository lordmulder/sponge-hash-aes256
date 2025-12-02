// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crate::{
    arguments::Args,
    common::{get_env, parse_enum, Flag},
};
use crossbeam_channel::{bounded, Sender};
use std::{
    borrow::Cow,
    collections::BTreeSet,
    fs::{self, DirEntry, Metadata},
    io::Write,
    iter,
    path::PathBuf,
    sync::{atomic::Ordering, Arc, OnceLock},
    thread,
};

/// Error type for processing files
#[derive(Debug)]
#[allow(dead_code)]
enum ErrorType {
    DirOpenError(PathBuf),
    DirReadError(PathBuf),
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

/// Enable breadth-first search?
#[inline]
fn parse_search_strategy() -> bool {
    static SEARCH_STRATEGY: OnceLock<bool> = OnceLock::new();
    *SEARCH_STRATEGY.get_or_init(|| {
        let strategy = get_env("SPONGE256SUM_DIRWALK_STRATEGY").and_then(|value| parse_enum(value, &["BFS", "DFS"]));
        strategy.map(|pos| pos == 0usize).unwrap_or(true)
    })
}

// ---------------------------------------------------------------------------
// Iterate input files/directories
// ---------------------------------------------------------------------------

type PathResult = Result<PathBuf, ErrorType>;

/// Iterate all files and sub-directories in a directory
#[allow(clippy::unnecessary_map_or)]
fn process_directory(path_tx: &Sender<PathResult>, path: &PathBuf, visited: &FileIdSet, args: &Arc<Args>, halt: &Arc<Flag>) -> bool {
    let dir_iter = match fs::read_dir(path) {
        Ok(dir_iter) => dir_iter,
        Err(_) => {
            let _ = path_tx.send(Err(ErrorType::DirOpenError(path.clone())));
            return false;
        }
    };

    let breadth_first = if args.recursive { parse_search_strategy() } else { false };
    let mut dir_queue = if breadth_first { Vec::with_capacity(32usize) } else { Vec::new() };

    for element in dir_iter {
        if halt.load(Ordering::Relaxed) {
            return false;
        }
        match element {
            Ok(dir_entry) => {
                if let Some(meta_data) = is_directory(&dir_entry) {
                    if args.recursive {
                        let file_id = file_id::get(&meta_data);
                        if file_id.map_or(true, |id| !visited.contains(&id)) {
                            if breadth_first {
                                dir_queue.push((file_id, dir_entry.path()));
                            } else if !(process_directory(path_tx, &dir_entry.path(), &append(visited, file_id), args, halt) || args.keep_going) {
                                return false;
                            }
                        }
                    }
                } else if path_tx.send(Ok(dir_entry.path())).is_err() {
                    return false;
                }
            }
            Err(_) => {
                let _ = path_tx.send(Err(ErrorType::DirReadError(path.clone())));
                return false;
            }
        }
    }

    if breadth_first {
        for (file_id, dir_name) in dir_queue.into_iter() {
            if !(process_directory(path_tx, &dir_name, &append(visited, file_id), args, halt) || args.keep_going) {
                return false;
            }
        }
    }

    true
}

/// Iterate a list of input files
fn iterate_files(path_tx: &Sender<PathResult>, args: &Arc<Args>, halt: &Arc<Flag>) {
    let handle_dirs = args.dirs || args.recursive;

    for file_name in args.files.iter().cloned() {
        if halt.load(Ordering::Relaxed) {
            break;
        }
        let dir_info = if handle_dirs { fs::metadata(&file_name).ok().filter(|meta| meta.is_dir()) } else { None };
        if let Some(meta_data) = dir_info {
            let visited = file_id::get(&meta_data).map_or_else(FileIdSet::new, |dir_id| iter::once(dir_id).collect());
            if !(process_directory(path_tx, &file_name, &visited, args, halt) || args.keep_going) {
                break;
            }
        } else if path_tx.send(Ok(file_name)).is_err() {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// Process all input files
#[allow(dead_code)]
pub fn process_files(output: &mut impl Write, _digest_size: usize, args: &Arc<Args>, halt: &Arc<Flag>) -> bool {
    let (path_tx, path_rx) = bounded::<PathResult>(128usize);

    let (args_cloned, halt_cloned) = (Arc::clone(args), Arc::clone(halt));
    let thread_iter = thread::spawn(move || iterate_files(&path_tx, &args_cloned, &halt_cloned));

    while let Ok(path) = path_rx.recv() {
        if writeln!(output, "Path: {:?}", path).is_err() {
            break;
        }
    }

    drop(path_rx);

    thread_iter.join().expect("Failed to join 'iterate_files' thread!");
    true
}
