// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use anstream::AutoStream;
use std::{
    fs::File,
    io::{stderr, stdin, stdout, Read, Result as IoResult, StderrLock, StdinLock, StdoutLock, Write},
    path::Path,
    sync::{Mutex, MutexGuard},
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
// Source wrapper
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

// ---------------------------------------------------------------------------
// Output wrapper
// ---------------------------------------------------------------------------

enum StderrWrapper {
    Lock(StderrLock<'static>),
    Auto(AutoStream<StderrLock<'static>>),
}

pub struct OutStream {
    out: StdoutLock<'static>,
    err: StderrWrapper,
}

impl OutStream {
    pub fn initialize(no_color_support: bool) -> Self {
        let (stdout_lock, stderr_lock) = (stdout().lock(), stderr().lock());
        Self {
            out: stdout_lock,
            err: match no_color_support {
                true => StderrWrapper::Lock(stderr_lock),
                _ => StderrWrapper::Auto(AutoStream::auto(stderr_lock)),
            },
        }
    }

    #[inline(always)]
    pub const fn out(&mut self) -> &mut dyn Write {
        &mut self.out
    }

    #[inline(always)]
    pub const fn err(&mut self) -> &mut dyn Write {
        match &mut self.err {
            StderrWrapper::Lock(stderr_lock) => stderr_lock,
            StderrWrapper::Auto(auto_stream) => auto_stream,
        }
    }
}
