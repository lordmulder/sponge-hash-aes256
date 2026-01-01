// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use std::{env, num::NonZeroUsize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

pub struct InvalidValue {
    pub name: String,
    pub value: String,
}

impl InvalidValue {
    pub fn new(name: &str, value: String) -> Self {
        Self { name: name.to_owned(), value }
    }
}

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

pub struct Env {
    pub dirwalk_strategy: Option<bool>,
    pub thread_count: Option<usize>,
    pub sefltest_passes: Option<NonZeroUsize>,
}

impl Env {
    pub fn from_env() -> Result<Self, InvalidValue> {
        Ok(Self {
            dirwalk_strategy: parse_enum("SPONGE256SUM_DIRWALK_STRATEGY", &["BFS", "DFS"])?.map(|index| index == 0usize),
            thread_count: parse_usize("SPONGE256SUM_THREAD_COUNT")?,
            sefltest_passes: parse_nonzero_usize("SPONGE256SUM_SELFTEST_PASSES")?,
        })
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

#[inline]
fn get_env(name: &str) -> Option<String> {
    env::var(name).ok().as_ref().map(|str| str.trim_ascii()).filter(|str| !str.is_empty()).map(str::to_string)
}

#[inline]
fn parse_usize(name: &str) -> Result<Option<usize>, InvalidValue> {
    match get_env(name) {
        Some(value) => match value.parse::<usize>() {
            Ok(value) => Ok(Some(value)),
            Err(_) => Err(InvalidValue::new(name, value)),
        },
        None => Ok(None),
    }
}

#[inline]
fn parse_nonzero_usize(name: &str) -> Result<Option<NonZeroUsize>, InvalidValue> {
    match get_env(name) {
        Some(value) => match value.parse::<usize>().ok().and_then(NonZeroUsize::new) {
            Some(value) => Ok(Some(value)),
            None => Err(InvalidValue::new(name, value)),
        },
        None => Ok(None),
    }
}

#[inline]
fn parse_enum(name: &str, options: &[&str]) -> Result<Option<usize>, InvalidValue> {
    match get_env(name) {
        Some(value) => match options.iter().position(|str| value.eq_ignore_ascii_case(str)) {
            Some(index) => Ok(Some(index)),
            None => Err(InvalidValue::new(name, value)),
        },
        None => Ok(None),
    }
}
