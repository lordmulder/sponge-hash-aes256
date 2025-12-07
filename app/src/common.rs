// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};
use tinyvec::TinyVec;

// ---------------------------------------------------------------------------
// Common definitions
// ---------------------------------------------------------------------------

/// Maximum allowable level of snailyness
pub const MAX_SNAIL_LEVEL: u8 = 4u8;

/// Maximum allowable digest size, specified in bytes
pub const MAX_DIGEST_SIZE: usize = 8usize * DEFAULT_DIGEST_SIZE;

/// Maximum number of threads
pub const MAX_THREADS: usize = 64usize;

/// Type for holding a digest
pub type Digest = TinyVec<[u8; DEFAULT_DIGEST_SIZE]>;

/// Error type to indicate that a process was aborted
pub struct Aborted;

// ---------------------------------------------------------------------------
// Cancellation flag
// ---------------------------------------------------------------------------

/// A flag which can be used to signal a cancellation request
pub struct Flag(AtomicUsize);

/// An error type indicating that the process could not be stopped, because it was already aborted
pub struct AlreadyAborted;

// Status constants
const STATUS_RUNNING: usize = 0usize;
const STATUS_STOPPED: usize = 1usize;
const STATUS_ABORTED: usize = 2usize;

impl Flag {
    #[inline(always)]
    pub fn cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed) != STATUS_RUNNING
    }

    #[inline(always)]
    pub fn stop_process(&self) -> Result<(), AlreadyAborted> {
        match self.0.compare_exchange(STATUS_RUNNING, STATUS_STOPPED, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => Ok(()),
            Err(STATUS_STOPPED) => Ok(()),
            Err(_) => Err(AlreadyAborted),
        }
    }

    #[inline(always)]
    pub fn abort_process(&self) {
        self.0.store(STATUS_ABORTED, Ordering::SeqCst);
    }
}

impl Default for Flag {
    #[inline]
    fn default() -> Self {
        Self(AtomicUsize::new(STATUS_RUNNING))
    }
}

// ---------------------------------------------------------------------------
// Detect number of CPU cores
// ---------------------------------------------------------------------------

/// Map the number of available CPU cores to the number of threads
///
/// **Note:** This avoids running too many parallel threads on systems with a large number of CPU cores!
fn cores_to_threads(cores: usize) -> NonZeroUsize {
    NonZeroUsize::new(((2.0 * (cores as f64).log2()).floor() as usize).max(1usize)).unwrap()
}

/// Get the "optimal" number of parallel threads for the current system
pub fn hardware_concurrency() -> NonZeroUsize {
    cores_to_threads(num_cpus::get())
}

// ---------------------------------------------------------------------------
// TinyVec extension
// ---------------------------------------------------------------------------

pub trait TinyVecEx {
    fn with_size(length: usize) -> Self;
}

impl<const N: usize> TinyVecEx for TinyVec<[u8; N]> {
    #[inline(always)]
    fn with_size(length: usize) -> Self {
        if length > N {
            let mut digest = Self::with_capacity(length);
            digest.resize(length, 0u8);
            digest
        } else {
            Self::from_array_len([0u8; N], length)
        }
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn increment(counter: &mut u64) {
    *counter = counter.saturating_add(1u64);
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
