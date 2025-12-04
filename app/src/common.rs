// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::{num::NonZeroUsize, sync::atomic::AtomicBool};
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

/// Atomic flag
pub type Flag = AtomicBool;

/// Error type to indicate that a process was aborted
pub struct Aborted;

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
// TinyVec helper
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn calloc_vec<const N: usize>(length: usize) -> TinyVec<[u8; N]> {
    let mut digest = TinyVec::with_capacity(length);
    digest.resize(length, 0u8);
    digest
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
