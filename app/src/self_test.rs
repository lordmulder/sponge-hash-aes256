// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use hex_literal::hex;
use rand_pcg::{
    Pcg64Mcg,
    rand_core::{RngCore, SeedableRng},
};
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};
use std::{io::Write, str::from_utf8, time::Instant};

use crate::{
    arguments::{Args, HEADER_LINE},
    check_running,
    common::{Error, Flag},
    digest::digest_equal,
    print_error,
};

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

fn print_digest<T: AsRef<[u8]>>(output: &mut impl Write, prefix: &str, digest: T) -> Result<(), Error> {
    assert!(digest.as_ref().len() <= DEFAULT_DIGEST_SIZE, "Length of digest exceeds capacity!");

    let mut hex_buffer = [0u8; DEFAULT_DIGEST_SIZE * 2usize];
    let hex_str = &mut hex_buffer[..digest.as_ref().len().checked_mul(2usize).unwrap()];

    encode_to_slice(digest, hex_str).unwrap();
    Ok(writeln!(output, "{prefix} {}", from_utf8(hex_str).unwrap())?)
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

const DIGEST_SIZE: usize = DEFAULT_DIGEST_SIZE;

const PCG64_SEEDVALUE: [u64; 2usize] = [18446744073709551557u64, 18446744073709551533u64];
const DIGEST_EXPECTED: [[u8; DIGEST_SIZE]; 2usize] =
    [hex!("721a31e8bafb3ed328459f8e87068283b7d19bc736469d02916355ce726873bf"), hex!("cf0bc20b6cc6e9268d0e91d3198ca631bdfc343f8f972bb21c2d3ed375acf1a4")];

const BUFFER_SIZE: usize = 4093usize;
const MAX_ITERATION: u32 = 524287u32;

fn do_test(seed: u64, digest_expected: &[u8; DIGEST_SIZE], output: &mut impl Write, counter: &mut u64, running: &Flag) -> Result<bool, Error> {
    let mut source = Pcg64Mcg::seed_from_u64(seed);
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut hasher = SpongeHash256::default();

    for _ in 0..MAX_ITERATION {
        source.fill_bytes(&mut buffer);
        hasher.update(buffer);
        *counter += buffer.len() as u64;
        check_running!(running);
    }

    let digest_computed: [u8; DIGEST_SIZE] = hasher.digest();
    let success = digest_equal(&digest_computed, digest_expected);

    if !success {
        writeln!(output, "Failure !!!\n")?;
        print_digest(output, "Computed:", digest_computed)?;
        print_digest(output, "Expected:", digest_expected)?;
    }

    Ok(success)
}

fn test_runner(output: &mut impl Write, running: Flag) -> Result<bool, Error> {
    writeln!(output, "{}\n", HEADER_LINE)?;
    writeln!(output, "Self-test is running, please be patient...")?;
    output.flush()?;

    let start_time = Instant::now();
    let mut total = 0u64;

    for (seed_value, digest_expected) in PCG64_SEEDVALUE.iter().zip(DIGEST_EXPECTED.iter()) {
        if !do_test(*seed_value, digest_expected, output, &mut total, &running)? {
            return Ok(false);
        }
    }

    assert_eq!(total, (BUFFER_SIZE as u64) * (MAX_ITERATION as u64) * (PCG64_SEEDVALUE.len() as u64));

    let elapsed = start_time.elapsed().as_secs_f64();
    let (rate, unit) = format_bytes((total as f64) / elapsed);

    writeln!(output, "Successful.\n")?;
    writeln!(output, "Completed in {:.1} seconds ({:.2} {}/s).", elapsed, rate, unit)?;

    Ok(true)
}

pub fn self_test(output: &mut impl Write, args: &Args, running: Flag) -> bool {
    match test_runner(output, running) {
        Err(Error::Aborted) => {
            print_error!(args, "Aborted: The process has been interrupted by the user!");
            false
        }
        Err(error) => {
            print_error!(args, "Self-test encountered an error: {:?}", error);
            false
        }
        Ok(result) => result,
    }
}
