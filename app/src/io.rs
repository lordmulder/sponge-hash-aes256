// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use std::sync::{Mutex, MutexGuard};
use std::{
    fs::File,
    io::{stdin, Read, Result as IoResult, StdinLock},
    path::Path,
};

use crate::os::STDIN_NAME;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    AccessDenied,
    IsADirectory,
}

// ---------------------------------------------------------------------------
// I/O wrapper
// ---------------------------------------------------------------------------

static STDIN_MUTEX: Mutex<()> = Mutex::new(());

pub enum DataSource<'a> {
    File(File),
    Stream((StdinLock<'a>, MutexGuard<'a, ()>)),
}

impl DataSource<'_> {
    pub fn from_stdin() -> Self {
        let guard = STDIN_MUTEX.try_lock().expect("Failed to lock 'stdin' stream!");
        Self::Stream((stdin().lock(), guard))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        if !STDIN_NAME.eq(path.as_ref()) {
            match File::open(path) {
                Ok(file) => {
                    if !Self::is_directory(&file) {
                        Ok(Self::File(file))
                    } else {
                        Err(Error::IsADirectory)
                    }
                }
                Err(io_error) => match io_error.kind() {
                    std::io::ErrorKind::NotFound => Err(Error::FileNotFound),
                    std::io::ErrorKind::IsADirectory => Err(Error::IsADirectory),
                    _ => Err(Error::AccessDenied),
                },
            }
        } else {
            Ok(Self::from_stdin())
        }
    }

    #[inline]
    fn is_directory(file: &File) -> bool {
        file.metadata().is_ok_and(|meta| meta.is_dir())
    }
}

impl Read for DataSource<'_> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self {
            DataSource::File(file) => file.read(buf),
            DataSource::Stream(stream) => stream.0.read(buf),
        }
    }
}
