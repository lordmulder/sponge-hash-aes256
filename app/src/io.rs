// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use lazy_static::lazy_static;
use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard};
use std::{
    fs::File,
    io::{stdin, Error as IoError, Read, Result as IoResult, StdinLock},
    path::Path,
};

use crate::common::Error;

// ---------------------------------------------------------------------------
// Standard streams
// ---------------------------------------------------------------------------

#[cfg(not(target_family = "windows"))]
lazy_static! {
    /// Standard input sentinal value for various OS
    pub static ref STDIN_NAME: OsString = OsString::from("/dev/stdin");
}

#[cfg(target_family = "windows")]
lazy_static! {
    /// Standard input sentinal value for various OS
    pub static ref STDIN_NAME: OsString = OsString::from("CON");
}

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
            Ok(guard) => Ok(DataSource::Stream((guard, stdin().lock()))),
            Err(_) => Err(Error::Io(IoError::other("Failed to lock 'stdin' handle, already in use!"))),
        }
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        if !STDIN_NAME.eq(path.as_ref()) {
            Ok(DataSource::File(File::open(path)?))
        } else {
            DataSource::from_stdin()
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            DataSource::File(file) => file.metadata().is_ok_and(|meta| meta.is_dir()),
            DataSource::Stream(_) => false,
        }
    }
}

impl Read for DataSource<'_> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let reader: &mut dyn Read = match self {
            DataSource::File(file) => file,
            DataSource::Stream(stream) => &mut stream.1,
        };
        reader.read(buf)
    }
}
