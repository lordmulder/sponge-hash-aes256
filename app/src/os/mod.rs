// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_family = "unix")] {
        mod unix;
        pub use unix::*;
    } else if #[cfg(target_family = "windows")] {
        mod windows;
        pub use windows::*;
    } else {
        compile_error!("Platform not currently supported!");
    }
}

cfg_if! {
    if #[cfg(target_pointer_width = "64")] {
        pub const IO_READ_BUFFER_SIZE: usize = 16384usize;
    } else if #[cfg(target_pointer_width = "32")] {
        pub const IO_READ_BUFFER_SIZE: usize = 8192usize;
    } else {
        compile_error!("Platform not currently supported!");
    }
}
