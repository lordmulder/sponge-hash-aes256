// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use hex_literal::hex;
use rand_pcg::{
    rand_core::{RngCore, SeedableRng},
    Pcg64Mcg,
};
use rolling_median::Median;
use sponge_hash_aes256::{SpongeHash256, DEFAULT_DIGEST_SIZE};
use std::{
    hint::black_box,
    io::{Error as IoError, Write},
    num::NonZeroUsize,
    time::Instant,
};

use crate::{
    arguments::{Args, HEADER_LINE},
    common::{Aborted, Flag},
    digest::digest_equal,
    environment::Env,
    print_error,
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum Error {
    Cancelled,
    IoError,
}

impl From<IoError> for Error {
    fn from(_io_error: IoError) -> Self {
        Self::IoError
    }
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

/// Convert the given "raw" number of bytes to the proper binary units
fn format_bytes(mut value: f64) -> (f64, &'static str) {
    const BIN_UNITS: [&str; 5usize] = ["Byte", "KiB", "MiB", "GiB", "TiB"];
    const MAX_INDEX: usize = BIN_UNITS.len() - 1usize;

    let mut index = 0usize;
    while (index < MAX_INDEX) && (value + f64::EPSILON > 999.9) {
        value /= 1024.0;
        index += 1usize;
    }

    (value, BIN_UNITS[index])
}

/// Format the given digest as hex string
fn format_digest<T: AsRef<[u8]>>(digest: T, hex_buffer: &mut [u8]) -> &str {
    let hex_len = digest.as_ref().len().checked_mul(2usize).unwrap();
    assert!(hex_buffer.len() >= hex_len, "Digest hex length exceeds buffer capacity!");
    encode_to_slice(digest, &mut hex_buffer[..hex_len]).unwrap();
    str::from_utf8(&hex_buffer[..hex_len]).expect("Failed to format digest!")
}

/// Check if the computation has been aborted
macro_rules! check_cancelled {
    ($halt:ident) => {
        if !$halt.running() {
            return Err(Error::Cancelled);
        }
    };
}

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

const PCG64_SEEDVALUE: [u64; 2usize] = [18446744073709551557u64, 18446744073709551533u64];
const DIGEST_EXPECTED: [[u8; DEFAULT_DIGEST_SIZE]; 2usize] =
    [hex!("fbb2f74509d78f4ac30da4a9ed0769efff7fbe5367e363b75572820b8aa83fe0"), hex!("87dac84f3f485a61bc6cb73f5cf236d68831c7bb8a0cef15cce500cf17a5690e")];

const BUFFER_SIZE: usize = 4093usize;
const MAX_ITERATION: u32 = 249989u32;

const TOTAL_BYTES: u64 = (BUFFER_SIZE as u64) * (MAX_ITERATION as u64) * (PCG64_SEEDVALUE.len() as u64);

/// The actual **SpongeHash256** self-test routine
fn do_self_test(output: &mut impl Write, halt: &Flag) -> Result<bool, Error> {
    let mut success = true;
    let mut counter = 0u64;

    for (seed_value, digest_expected) in PCG64_SEEDVALUE.iter().zip(DIGEST_EXPECTED.iter()) {
        let mut source = Pcg64Mcg::seed_from_u64(*seed_value);
        let mut buffer = [0u8; BUFFER_SIZE];
        let mut hasher = SpongeHash256::default();

        for _ in 0..MAX_ITERATION {
            source.fill_bytes(&mut buffer);
            hasher.update(buffer);
            counter += buffer.len() as u64;
            check_cancelled!(halt);
        }

        let digest_computed: [u8; DEFAULT_DIGEST_SIZE] = black_box(hasher.digest());

        if cfg!(debug_assertions) {
            let mut hex_buffer = [0u8; DEFAULT_DIGEST_SIZE * 2usize];
            writeln!(output, "Computed: {}", format_digest(digest_computed, &mut hex_buffer))?;
            writeln!(output, "Expected: {}", format_digest(digest_expected, &mut hex_buffer))?;
        }

        success &= digest_equal(&digest_computed, digest_expected);
    }

    assert_eq!(counter, TOTAL_BYTES);
    Ok(success)
}

/// Runs the self-test routine for `passes` times
fn test_runner(output: &mut impl Write, passes: NonZeroUsize, args: &Args, halt: &Flag) -> Result<bool, Error> {
    writeln!(output, "{}", HEADER_LINE)?;
    let mut median = Median::new();

    for pass in 0usize..passes.get() {
        writeln!(output, "\nSelf-test pass {} of {} is running...", (pass as u32) + 1u32, passes)?;
        output.flush()?;
        check_cancelled!(halt);

        let start_time = Instant::now();
        let success = black_box(do_self_test(output, halt)?);
        let elapsed = start_time.elapsed();

        writeln!(output, "{}", if success { "Successful." } else { "Failure !!!" })?;

        if !(success || args.keep_going) {
            return Ok(false);
        }

        median.push(elapsed.as_secs_f64()).expect("Invalid elapsed time!");
    }

    let secs_median = median.get().unwrap_or(f64::MAX);
    let (rate_median, rate_unit) = format_bytes((TOTAL_BYTES as f64) / secs_median);

    writeln!(output, "\n--------\n")?;
    writeln!(output, "Median execution time: {:.1} seconds ({:.2} {}/s)", secs_median, rate_median, rate_unit)?;

    Ok(true)
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

/// The built-in self-test (BIST)
pub fn self_test(output: &mut impl Write, args: &Args, env: &Env, halt: &Flag) -> Result<bool, Aborted> {
    let passes = env.sefltest_passes.unwrap_or(NonZeroUsize::new(3usize).unwrap());

    match test_runner(output, passes, args, halt) {
        Ok(result) => Ok(result),
        Err(Error::Cancelled) => Err(Aborted),
        Err(error) => {
            print_error!(args, "Self-test encountered an error: {:?}", error);
            Ok(false)
        }
    }
}
