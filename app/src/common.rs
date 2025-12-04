// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use std::{num::NonZeroUsize, sync::atomic::AtomicBool};

// ---------------------------------------------------------------------------
// Common definitions
// ---------------------------------------------------------------------------

/// Maximum allowable level of snailyness
pub const MAX_SNAIL_LEVEL: u8 = 4u8;

/// Maximum allowable digest size, specified in bytes
pub const MAX_DIGEST_SIZE: usize = 64usize;

/// Maximum number of threads
pub const MAX_THREADS: usize = 64usize;

/// Atomic flag
pub type Flag = AtomicBool;

/// Type for holding a digest
pub type Digest = [u8; MAX_DIGEST_SIZE];
pub const EMPTY_DIGEST: Digest = [0u8; MAX_DIGEST_SIZE];

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
