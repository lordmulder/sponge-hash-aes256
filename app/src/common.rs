// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use std::{fmt::Display, io::Error as IoError, sync::mpsc::Receiver};

// ---------------------------------------------------------------------------
// Common definitions
// ---------------------------------------------------------------------------

/// Maximum allowable level of snailyness
pub const MAX_SNAIL_LEVEL: u8 = 4u8;

/// Maximum allowable digest size, specified in bytes
pub const MAX_DIGEST_SIZE: usize = 256usize;

/// Cancellation flag
pub type Flag = Receiver<bool>;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

pub enum Error {
    Aborted,
    Io(IoError),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, f),
            Self::Aborted => write!(f, "Interrupted by user!"),
        }
    }
}

// ---------------------------------------------------------------------------
// Helper macros
// ---------------------------------------------------------------------------

/// Conditional printing of error message
#[macro_export]
macro_rules! print_error {
    ($args:ident, $fmt:literal $(,$arg:expr)*$(,)?) => {
        if !$args.quiet {
            eprintln!(concat!("[sponge256sum] ", $fmt) $(, $arg)*);
        }
    };
}

/// Unified error handling routine
#[macro_export]
macro_rules! handle_error {
    ($args:ident, $err_counter:ident, $($message:tt)*) => {{
        print_error!($args, $($message)*);
        if $args.keep_going {
            *$err_counter += 1usize;
        } else {
            return false;
        }
    }};
}

/// Check whether the process has been interrupted
#[macro_export]
macro_rules! check_running {
    ($channel:ident) => {
        if $channel.try_recv().unwrap_or_default() {
            return Err(Error::Aborted);
        }
    };
    ($args:ident, $channel:ident) => {
        if $channel.try_recv().unwrap_or_default() {
            $crate::print_error!($args, "Aborted: The process has been interrupted by the user!");
            return false;
        }
    };
}
