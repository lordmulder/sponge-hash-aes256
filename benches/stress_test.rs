// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute as compute_hash};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    process,
    time::Instant,
};

// ---------------------------------------------------------------------------
// Stress test
// ---------------------------------------------------------------------------

const UPDATE_CYCLE: u64 = 997u64;

fn stress_test() {
    let mut digest: [u8; DEFAULT_DIGEST_SIZE];
    let mut hex_buffer = [0u8; 64usize];
    let mut set: HashSet<[u8; DEFAULT_DIGEST_SIZE]> = HashSet::new();
    let mut counter = 0u64;

    let file = match File::open(Path::new("benches").join("data").join("input.txt")) {
        Ok(file_handle) => file_handle,
        Err(error) => {
            eprintln!("Failed to open input file 'input.txt' for reading: {:?}", error);
            process::exit(1);
        }
    };

    for line in BufReader::new(file).lines() {
        match line {
            Ok(content) => {
                let item = content.trim();
                if !item.is_empty() {
                    digest = compute_hash(item.as_bytes());
                    counter += 1u64;
                    let success = set.insert(digest);
                    if (!success) || (counter % UPDATE_CYCLE == 0u64) {
                        if encode_to_slice(&digest, &mut hex_buffer).is_ok() {
                            let digest_string = unsafe { str::from_utf8_unchecked(&hex_buffer) };
                            println!("[{:0>8}] {} << {:?}", counter, digest_string, item);
                        }
                    }
                    if !success {
                        eprintln!("Colission has been detected!");
                        eprintln!("The value that caused the colission is: {:?}", item);
                        process::exit(1);
                    }
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

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const NUM_RUNS: usize = 3usize;

fn main() {
    let mut fastest_run = f64::MAX;

    for _ in 0usize..NUM_RUNS {
        let start_time = Instant::now();
        stress_test();
        fastest_run = fastest_run.min(start_time.elapsed().as_secs_f64());
    }

    println!("--------");
    println!("Execution took {:.1} seconds.", fastest_run);
}
