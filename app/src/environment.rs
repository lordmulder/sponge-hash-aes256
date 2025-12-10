// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use std::{
    env,
    fmt::{Display, Formatter, Result as FmtResult},
    num::NonZeroU16,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

pub struct InvalidValue(String);

impl Display for InvalidValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

fn get_env(name: &str) -> Option<String> {
    env::var(name).ok().as_ref().map(|str| str.trim_ascii()).filter(|str| !str.is_empty()).map(str::to_string)
}

fn parse_enum(value: String, options: &[&str]) -> Result<usize, InvalidValue> {
    if value.is_empty() || options.is_empty() {
        Err(InvalidValue(value))
    } else {
        options.iter().position(|str| value.eq_ignore_ascii_case(str)).ok_or(InvalidValue(value))
    }
}

// ---------------------------------------------------------------------------
// Environment variables
// ---------------------------------------------------------------------------

/// The directory walking strategy
#[inline]
pub fn get_search_strategy() -> Result<Option<bool>, InvalidValue> {
    match get_env("SPONGE256SUM_DIRWALK_STRATEGY") {
        Some(str) => parse_enum(str, &["BFS", "DFS"]).map(|index| Some(index == 0usize)),
        None => Ok(None),
    }
}

/// The number of threads for multi-threaded processing
#[inline]
pub fn get_thread_count() -> Result<Option<usize>, InvalidValue> {
    match get_env("SPONGE256SUM_THREAD_COUNT") {
        Some(str) => str.parse::<usize>().map(Some).map_err(|_| InvalidValue(str)),
        None => Ok(None),
    }
}

/// The number of threads for multi-threaded processing
#[inline]
pub fn get_selftest_passes() -> Result<Option<NonZeroU16>, InvalidValue> {
    match get_env("SPONGE256SUM_SELFTEST_PASSES") {
        Some(str) => str.parse::<NonZeroU16>().map(Some).map_err(|_| InvalidValue(str)),
        None => Ok(None),
    }
}
