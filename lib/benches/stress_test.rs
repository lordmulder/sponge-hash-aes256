// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use rolling_median::Median;
use sponge_hash_aes256::{compute as compute_hash, DEFAULT_DIGEST_SIZE};
use std::{
    borrow::Cow,
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process,
    time::{Duration, Instant},
};

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

macro_rules! trim_env {
    ($name:expr) => {
        option_env!($name).map(str::trim).filter(|str| !str.is_empty())
    };
}

// ---------------------------------------------------------------------------
// Stress test
// ---------------------------------------------------------------------------

const PROGRESS_UPDATE_CYCLE: usize = 99839usize;

fn stress_test() -> Duration {
    let mut set: HashSet<[u8; DEFAULT_DIGEST_SIZE]> = HashSet::new();
    let mut counter = 1usize;

    let input_file_name: Cow<Path> = match trim_env!("SPONGE_BENCH_SOURCE") {
        Some(path) => Path::new(path).into(),
        _ => Path::new(env!("CARGO_MANIFEST_DIR")).join("benches").join("data").join("input.txt").into(),
    };

    let file = match File::open(&input_file_name) {
        Ok(file_handle) => file_handle,
        Err(error) => {
            eprintln!("Failed to open input file {input_file_name:?} for reading: {:?}", error);
            process::exit(1);
        }
    };

    let start_time = Instant::now();

    for line in BufReader::new(file).lines() {
        match line {
            Ok(content) => {
                let item = content.trim();
                if !item.is_empty() {
                    process_input(&mut set, &mut counter, item);
                }
            }
            Err(error) => {
                eprintln!("Failed to reading next line: {:?}", error);
                process::exit(1);
            }
        }
    }

    let time_elapsed = start_time.elapsed();

    println!("All items inserted.");
    println!("Total number of unique items is: {}", set.len());

    time_elapsed
}

#[inline(always)]
fn process_input(set: &mut HashSet<[u8; DEFAULT_DIGEST_SIZE]>, counter: &mut usize, input: &str) {
    let digest = compute_hash(None, input.as_bytes());
    let success = set.insert(digest);
    if (!success) || (*counter >= PROGRESS_UPDATE_CYCLE) {
        let mut hex_buffer = [0u8; 64usize];
        if encode_to_slice(digest, &mut hex_buffer).is_ok() {
            let digest_string = unsafe { str::from_utf8_unchecked(&hex_buffer) };
            println!("[{:0>9}] {} << {:?}", set.len(), digest_string, input);
        }
        *counter = 0usize;
    }
    if !success {
        eprintln!("Collision has been detected!");
        eprintln!("The value that caused the collision is: {:?}", input);
        process::exit(1);
    } else {
        *counter += 1usize;
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let passes = trim_env!("SPONGE_BENCH_PASSES").and_then(|str| str.parse().ok()).filter(|val| *val >= 1u16).unwrap_or(3u16);
    let mut rolling_median = Median::new();

    for _i in 0u16..passes {
        assert!(rolling_median.push(stress_test().as_secs_f64()).is_ok());
        println!("--------");
    }

    println!("Median execution time: {:.2} seconds.", rolling_median.get().unwrap_or(f64::MAX));
}
