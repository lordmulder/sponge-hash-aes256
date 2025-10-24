// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::Receiver;
use std::{
    collections::HashMap,
    env,
    fmt::Display,
    io::Error as IoError,
    sync::{LazyLock, Mutex},
};

// ---------------------------------------------------------------------------
// Common definitions
// ---------------------------------------------------------------------------

/// Maximum allowable level of snailyness
pub const MAX_SNAIL_LEVEL: u8 = 4u8;

/// Maximum allowable digest size, specified in bytes
pub const MAX_DIGEST_SIZE: usize = 256usize;

/// Cancellation flag
pub type Flag = Receiver<()>;

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

type EnvValueType = Option<&'static str>;

static ENV_MAP: LazyLock<Mutex<HashMap<&'static str, EnvValueType>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

fn init_env(name: &str) -> Option<&'static str> {
    match env::var(name).ok().as_ref().map(|str| str.trim_ascii()) {
        Some(str) if !str.is_empty() => Some(Box::leak(Box::new(str.to_owned()))),
        _ => None,
    }
}

pub fn get_env(name: &'static str) -> EnvValueType {
    let mut guard = ENV_MAP.lock().unwrap();
    *guard.entry(name).or_insert_with(|| init_env(name))
}

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
    ($channel_rx:ident) => {
        if $channel_rx.try_recv().is_ok() {
            return Err(Error::Aborted);
        }
    };
    ($args:ident, $channel_rx:ident) => {
        if $channel_rx.try_recv().is_ok() {
            $crate::abort!($args)
        }
    };
}

/// Abort process, e.g., after it has been interrupted
#[macro_export]
macro_rules! abort {
    ($args:ident) => {{
        $crate::print_error!($args, "Aborted: The process has been interrupted by the user!");
        std::process::exit(130i32);
    }};
}
