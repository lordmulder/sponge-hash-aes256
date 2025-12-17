// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use num::traits::SaturatingAdd;
use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::{
    num::NonZeroUsize,
    sync::atomic::{AtomicUsize, Ordering},
};
use tinyvec::{ArrayVec, TinyVec};

// ---------------------------------------------------------------------------
// Common definitions
// ---------------------------------------------------------------------------

/// Maximum allowable "snailyness" (throttling) level
pub const MAX_SNAIL_LEVEL: u8 = 4u8;

/// Maximum allowable digest size, specified in bytes
pub const MAX_DIGEST_SIZE: usize = 8usize * DEFAULT_DIGEST_SIZE;

/// Type for holding a digest
pub type Digest = TinyVec<[u8; DEFAULT_DIGEST_SIZE]>;

/// Error type to indicate that a process was aborted
pub struct Aborted;

// ---------------------------------------------------------------------------
// Cancellation flag
// ---------------------------------------------------------------------------

/// A flag which can be used to signal a cancellation request
pub struct Flag(AtomicUsize);

/// An error type indicating that the cancellation flag could not be updated
pub struct UpdateError;

// Status constants
const STATUS_RUNNING: usize = 0usize;
const STATUS_STOPPED: usize = 1usize;
const STATUS_ABORTED: usize = 2usize;

impl Flag {
    /// Check whether the process is still running
    ///
    /// This will return `true`, unless either `stop_process()` or `abort_process()` has been triggered.
    #[inline(always)]
    pub fn running(&self) -> bool {
        self.0.load(Ordering::Relaxed) == STATUS_RUNNING
    }

    /// Request the process to be stopped normally
    #[inline]
    pub fn stop_process(&self) -> Result<(), UpdateError> {
        self.try_update(STATUS_STOPPED)
    }

    /// Request the process to be aborted, e.g., after a `SIGINT` was received
    #[inline]
    pub fn abort_process(&self) -> Result<(), UpdateError> {
        self.try_update(STATUS_ABORTED)
    }

    #[inline(always)]
    fn try_update(&self, new_state: usize) -> Result<(), UpdateError> {
        match self.0.compare_exchange(STATUS_RUNNING, new_state, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => Ok(()),
            Err(previous_state) => match previous_state == new_state {
                true => Ok(()),
                false => Err(UpdateError),
            },
        }
    }
}

impl Default for Flag {
    #[inline]
    fn default() -> Self {
        Self(AtomicUsize::new(STATUS_RUNNING))
    }
}

// ---------------------------------------------------------------------------
// TinyVec extension
// ---------------------------------------------------------------------------

pub trait TinyVecEx {
    fn with_length(length: usize) -> Self;
}

impl<const N: usize, T: Copy + Default> TinyVecEx for TinyVec<[T; N]> {
    #[inline(always)]
    fn with_length(length: usize) -> Self {
        if length <= N {
            TinyVec::Inline(ArrayVec::from_array_len([T::default(); N], length))
        } else {
            TinyVec::Heap(vec![T::default(); length])
        }
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Increments the referenced counter by one (saturating)
#[inline(always)]
pub fn increment<T: SaturatingAdd + From<u8>>(counter: &mut T) {
    *counter = counter.saturating_add(&T::from(1u8));
}

/// Compute the thread-count-specific capacity for a bounded channel
#[inline]
pub fn get_capacity(thread_count: &NonZeroUsize) -> usize {
    let capacity = thread_count.get().saturating_mul(2usize).saturating_add(1usize);
    if capacity > isize::MAX as usize {
        capacity
    } else {
        capacity.next_power_of_two()
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
