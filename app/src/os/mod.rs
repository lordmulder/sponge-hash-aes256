// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

#[cfg(not(any(target_family = "unix", target_family = "windows")))]
compile_error!("Platform not currently supported!");
#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("Platform not currently supported!");

#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_family = "windows")]
mod windows;

#[cfg(target_pointer_width = "64")]
pub const IO_BUFFER_SIZE: usize = 16384usize;
#[cfg(target_pointer_width = "32")]
pub const IO_BUFFER_SIZE: usize = 8192usize;

#[cfg(target_family = "unix")]
pub use unix::*;
#[cfg(target_family = "windows")]
pub use windows::*;
