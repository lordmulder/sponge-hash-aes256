// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use hex_literal::hex;
use rand_pcg::{
    rand_core::{RngCore, SeedableRng},
    Pcg64Mcg,
};
use rolling_median::Median;
use sponge_hash_aes256::{SpongeHash256, DEFAULT_DIGEST_SIZE};
use std::{
    io::{Error as IoError, Result as IoResult, Write},
    num::NonZeroU16,
    str::from_utf8,
    time::Instant,
};

use crate::{
    arguments::{Args, HEADER_LINE},
    common::{Aborted, Flag},
    digest::digest_equal,
    environment::get_selftest_passes,
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

fn print_digest<T: AsRef<[u8]>>(output: &mut impl Write, prefix: &str, digest: T) -> IoResult<()> {
    assert!(digest.as_ref().len() <= DEFAULT_DIGEST_SIZE, "Digest length exceeds capacity!");

    let mut hex_buffer = [0u8; DEFAULT_DIGEST_SIZE * 2usize];
    let hex_str = &mut hex_buffer[..digest.as_ref().len().checked_mul(2usize).unwrap()];

    encode_to_slice(digest, hex_str).unwrap();
    writeln!(output, "{prefix} {}", from_utf8(hex_str).unwrap())
}

/// Check if the computation has been aborted
macro_rules! check_cancelled {
    ($halt:ident) => {
        if $halt.cancelled() {
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

fn do_test(seed: u64, digest_expected: &[u8; DEFAULT_DIGEST_SIZE], output: &mut impl Write, counter: &mut u64, halt: &Flag) -> Result<bool, Error> {
    let mut source = Pcg64Mcg::seed_from_u64(seed);
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut hasher = SpongeHash256::default();

    for _ in 0..MAX_ITERATION {
        source.fill_bytes(&mut buffer);
        hasher.update(buffer);
        *counter += buffer.len() as u64;
        check_cancelled!(halt);
    }

    let digest_computed: [u8; DEFAULT_DIGEST_SIZE] = hasher.digest();
    let success = digest_equal(&digest_computed, digest_expected);

    if !success {
        writeln!(output, "Failure !!!\n")?;
        print_digest(output, "Computed:", digest_computed)?;
        print_digest(output, "Expected:", digest_expected)?;
    }

    Ok(success)
}

fn test_runner(output: &mut impl Write, passes: NonZeroU16, halt: &Flag) -> Result<bool, Error> {
    writeln!(output, "{}\n", HEADER_LINE)?;
    let mut elapsed_median = Median::new();

    for i in 0u16..passes.get() {
        writeln!(output, "Self-test pass {} of {} is running...", (i as u32) + 1u32, passes)?;
        output.flush()?;

        let start_time = Instant::now();
        let mut total = 0u64;

        for (seed_value, digest_expected) in PCG64_SEEDVALUE.iter().zip(DIGEST_EXPECTED.iter()) {
            check_cancelled!(halt);
            if !do_test(*seed_value, digest_expected, output, &mut total, halt)? {
                return Ok(false);
            }
        }

        assert_eq!(total, TOTAL_BYTES);
        elapsed_median.push(start_time.elapsed().as_secs_f64());
        writeln!(output, "Successful.\n")?;
    }

    let secs_median = elapsed_median.get().unwrap_or(f64::MAX);
    let (rate_median, rate_unit) = format_bytes((TOTAL_BYTES as f64) / secs_median);

    writeln!(output, "--------\n")?;
    writeln!(output, "Median execution time: {:.1} seconds ({:.2} {}/s)", secs_median, rate_median, rate_unit)?;

    Ok(true)
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

pub fn self_test(output: &mut impl Write, args: &Args, halt: &Flag) -> Result<bool, Aborted> {
    let passes = match get_selftest_passes() {
        Ok(value) => value,
        Err(error) => {
            print_error!(args, "Error: Invalid number of self-test passes \"{}\" specified!", error);
            return Ok(false);
        }
    };

    match test_runner(output, passes, halt) {
        Ok(result) => Ok(result),
        Err(Error::Cancelled) => Err(Aborted),
        Err(error) => {
            print_error!(args, "Self-test encountered an error: {:?}", error);
            Ok(false)
        }
    }
}
