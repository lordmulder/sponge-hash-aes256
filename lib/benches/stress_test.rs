// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute as compute_hash};
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
// Stress test
// ---------------------------------------------------------------------------

const PROGRESS_UPDATE_CYCLE: usize = 9973usize;

fn stress_test() {
    let mut set: HashSet<[u8; DEFAULT_DIGEST_SIZE]> = HashSet::new();
    let mut counter = 1usize;

    let input_file_name: Cow<Path> = match option_env!("SPONGE_BENCH_INPUT_FILE").map(str::trim_ascii) {
        Some(path) if !path.is_empty() => Path::new(path).into(),
        _ => Path::new(env!("CARGO_MANIFEST_DIR")).join("benches").join("data").join("input.txt").into(),
    };

    let file = match File::open(&input_file_name) {
        Ok(file_handle) => file_handle,
        Err(error) => {
            eprintln!("Failed to open input file {input_file_name:?} for reading: {:?}", error);
            process::exit(1);
        }
    };

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

    println!("All items inserted.");
    println!("Total number of unique items is: {}", set.len());
}

#[inline(always)]
fn process_input(set: &mut HashSet<[u8; DEFAULT_DIGEST_SIZE]>, counter: &mut usize, input: &str) {
    let digest = compute_hash(None, input.as_bytes());
    let success = set.insert(digest);
    if (!success) || (*counter >= PROGRESS_UPDATE_CYCLE) {
        let mut hex_buffer = [0u8; 64usize];
        if encode_to_slice(&digest, &mut hex_buffer).is_ok() {
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

const NUM_RUNS: usize = 3usize;

fn main() {
    let mut measurements = [Duration::default(); NUM_RUNS];

    for i in 0usize..NUM_RUNS {
        let start_time = Instant::now();
        stress_test();
        measurements[i] = start_time.elapsed();
        println!("--------");
    }

    measurements.sort();
    let median = measurements[NUM_RUNS / 2usize];

    println!("Median execution time: {:.2} seconds.", median.as_secs_f64());
}
