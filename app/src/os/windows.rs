// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use std::{
    ffi::OsString,
    os::windows::io::{AsRawHandle, RawHandle},
    sync::LazyLock,
};
use windows_sys::Win32::Storage::FileSystem::{GetFileType, FILE_TYPE_PIPE};

use crate::io::DataSource;

// ---------------------------------------------------------------------------
// Windows functions
// ---------------------------------------------------------------------------

pub static STDIN_NAME: LazyLock<OsString> = LazyLock::new(|| OsString::from("CONIN$"));

pub fn is_pipe(data_source: &DataSource) -> bool {
    let file_type = unsafe { GetFileType(data_source.as_raw_handle()) };
    file_type == FILE_TYPE_PIPE
}

impl AsRawHandle for DataSource<'_> {
    #[inline(always)]
    fn as_raw_handle(&self) -> RawHandle {
        match self {
            DataSource::File(file) => file.as_raw_handle(),
            DataSource::Stream(stream) => stream.0.as_raw_handle(),
        }
    }
}
