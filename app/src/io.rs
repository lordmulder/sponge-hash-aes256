// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use std::ffi::OsString;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::{
    fs::File,
    io::{stdin, Read, Result as IoResult, StdinLock},
    path::Path,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum Error {
    Lock,
    Open,
}

// ---------------------------------------------------------------------------
// Standard streams
// ---------------------------------------------------------------------------

#[cfg(target_family = "windows")]
pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("CON"));

#[cfg(target_family = "unix")]
pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("/dev/stdin"));

// ---------------------------------------------------------------------------
// I/O wrapper
// ---------------------------------------------------------------------------

static STDIN_MUTEX: Mutex<()> = Mutex::new(());

pub enum DataSource<'a> {
    File(File),
    Stream((MutexGuard<'a, ()>, StdinLock<'a>)),
}

impl DataSource<'_> {
    pub fn from_stdin() -> Result<Self, Error> {
        match STDIN_MUTEX.try_lock() {
            Ok(guard) => Ok(Self::Stream((guard, stdin().lock()))),
            Err(_) => Err(Error::Lock),
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        if !STDIN_NAME.eq(path.as_ref()) {
            match File::open(path) {
                Ok(file) => Ok(Self::File(file)),
                Err(_) => Err(Error::Open),
            }
        } else {
            Self::from_stdin()
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            Self::File(file) => file.metadata().is_ok_and(|meta| meta.is_dir()),
            Self::Stream(_) => false,
        }
    }
}

impl Read for DataSource<'_> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self {
            DataSource::File(file) => file as &mut dyn Read,
            DataSource::Stream(stream) => &mut stream.1,
        }
        .read(buf)
    }
}
