// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crossbeam_channel::Receiver;
use lazy_static::lazy_static;
use std::{collections::HashMap, env, fmt::Display, io::Error as IoError, sync::Mutex};

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

lazy_static! {
    pub static ref ENV_CACHE: Mutex<HashMap<&'static str, Option<&'static str>>> = Mutex::new(HashMap::new());
}

#[inline(always)]
fn str_to_static(str: &str) -> &'static str {
    static EMPTY_STRING: &str = "";
    if !str.is_empty() {
        Box::leak(Box::new(str.to_owned()))
    } else {
        EMPTY_STRING
    }
}

pub fn get_env(name: &'static str) -> Option<&'static str> {
    let mut cache = ENV_CACHE.lock().unwrap();
    *cache.entry(name).or_insert_with(|| match env::var(name).ok().as_ref().map(|str| str.trim()) {
        Some(str) if !str.is_empty() => Some(str_to_static(str)),
        _ => None,
    })
}

pub fn parse_enum(value: &str, options: &[&str]) -> Option<usize> {
    if value.is_empty() || options.is_empty() {
        None
    } else {
        options.iter().position(|str| value.eq_ignore_ascii_case(str))
    }
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
