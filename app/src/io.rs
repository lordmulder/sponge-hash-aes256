// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use std::ffi::OsString;
use std::sync::{LazyLock, Mutex, MutexGuard, TryLockResult};
use std::thread;
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
    FileNotFound,
    AccessDenied,
    IsADirectory,
}

// ---------------------------------------------------------------------------
// Standard streams
// ---------------------------------------------------------------------------

#[cfg(target_family = "windows")]
pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("CON"));

#[cfg(target_family = "unix")]
pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("/dev/stdin"));

// ---------------------------------------------------------------------------
// Mutex helper
// ---------------------------------------------------------------------------

trait MutexEx<T: ?Sized> {
    fn try_lock_n(&self, retry: usize) -> TryLockResult<MutexGuard<'_, T>>;
}

impl<T: ?Sized> MutexEx<T> for Mutex<T> {
    #[inline]
    fn try_lock_n(&self, retry: usize) -> TryLockResult<MutexGuard<'_, T>> {
        for _ in 0usize..retry {
            if let Ok(guard) = self.try_lock() {
                return Ok(guard);
            } else {
                thread::yield_now();
            }
        }
        self.try_lock()
    }
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
    pub fn from_stdin() -> Self {
        match STDIN_MUTEX.try_lock_n(128usize) {
            Ok(guard) => Self::Stream((guard, stdin().lock())),
            Err(_) => panic!("Failed to lock 'stdin' stream!"),
        }
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
            DataSource::File(file) => file as &mut dyn Read,
            DataSource::Stream(stream) => &mut stream.1,
        }
        .read(buf)
    }
}
