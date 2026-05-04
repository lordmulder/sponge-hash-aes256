// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use libc::{fstat, stat};
use std::{
    ffi::OsString,
    fs::Metadata,
    mem::zeroed,
    os::{
        fd::{AsRawFd, RawFd},
        unix::fs::MetadataExt,
    },
    sync::LazyLock,
};

use crate::io::DataSource;

// ---------------------------------------------------------------------------
// Pipe functions
// ---------------------------------------------------------------------------

pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("/dev/stdin"));

pub fn is_pipe(data_source: &DataSource) -> bool {
    let mut info: stat = unsafe { zeroed() };

    if unsafe { fstat(data_source.as_raw_fd(), &mut info) } != 0 {
        return false; /*failure!*/
    }

    matches!(info.st_mode & libc::S_IFMT, libc::S_IFIFO | libc::S_IFSOCK)
}

impl AsRawFd for DataSource<'_> {
    #[inline(always)]
    fn as_raw_fd(&self) -> RawFd {
        match self {
            DataSource::File(file) => file.as_raw_fd(),
            DataSource::Stream(stream) => stream.0.as_raw_fd(),
        }
    }
}

// ---------------------------------------------------------------------------
// File id functions
// ---------------------------------------------------------------------------

pub type DevId = u64;
pub type InoId = u64;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct FileId {
    dev: DevId,
    ino: InoId,
}

impl FileId {
    #[inline]
    pub const fn new(dev: u64, ino: u64) -> Self {
        Self { dev, ino }
    }

    #[inline]
    pub fn dev(&self) -> DevId {
        self.dev
    }

    #[inline]
    pub fn same_dev(&self, other: DevId) -> bool {
        self.dev == other
    }
}

#[inline]
pub fn file_id(meta: Metadata) -> Option<FileId> {
    Some(FileId::new(meta.dev(), meta.ino()))
}
