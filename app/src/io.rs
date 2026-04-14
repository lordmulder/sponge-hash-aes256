// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use std::ffi::OsString;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::{
    fs::File,
    io::{stdin, Read, Result as IoResult, StdinLock},
    path::Path,
};

#[cfg(target_family = "unix")]
use std::os::fd::{AsRawFd, RawFd};
#[cfg(target_family = "windows")]
use std::os::windows::io::{AsRawHandle, RawHandle};

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
pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("CONIN$"));

#[cfg(not(target_family = "windows"))]
pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("/dev/stdin"));

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

#[cfg(target_family = "unix")]
impl AsRawFd for DataSource<'_> {
    #[inline(always)]
    fn as_raw_fd(&self) -> RawFd {
        match self {
            DataSource::File(file) => file.as_raw_fd(),
            DataSource::Stream(stream) => stream.0.as_raw_fd(),
        }
    }
}

#[cfg(target_family = "windows")]
impl AsRawHandle for DataSource<'_> {
    #[inline(always)]
    fn as_raw_handle(&self) -> RawHandle {
        match self {
            DataSource::File(file) => file.as_raw_handle(),
            DataSource::Stream(stream) => stream.0.as_raw_handle(),
        }
    }
}

// ---------------------------------------------------------------------------
// Utility Functions
// ---------------------------------------------------------------------------

#[cfg(target_family = "unix")]
pub fn is_pipe(data_source: &DataSource) -> bool {
    use libc::{fstat, stat};
    use std::mem;
    let mut info: stat = unsafe { mem::zeroed() };
    if unsafe { fstat(data_source.as_raw_fd(), &mut info) } == 0 {
        matches!(info.st_mode & libc::S_IFMT, libc::S_IFIFO | libc::S_IFSOCK)
    } else {
        false /*failure!*/
    }
}

#[cfg(target_family = "windows")]
pub fn is_pipe(data_source: &DataSource) -> bool {
    use windows_sys::Win32::Storage::FileSystem::{GetFileType, FILE_TYPE_PIPE};
    let file_type = unsafe { GetFileType(data_source.as_raw_handle()) };
    file_type == FILE_TYPE_PIPE
}

#[cfg(not(any(target_family = "unix", target_family = "windows")))]
pub fn is_pipe(_: &DataSource) -> bool {
    false
}
