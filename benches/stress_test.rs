// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use criterion::{Criterion, criterion_group, criterion_main};
use hex::encode;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

fn bench_stress(_: &mut Criterion) {
    let mut set: HashSet<[u8; DEFAULT_DIGEST_SIZE]> = HashSet::new();
    let mut counter = 0u64;
    let file = File::open("benches/data/input.txt").expect("Failed to open input file \"input.txt\" for reading!");

    for line in BufReader::new(file).lines() {
        match line {
            Ok(content) => {
                let item = content.trim();
                if !item.is_empty() {
                    let digest: [u8; DEFAULT_DIGEST_SIZE] = compute(item.as_bytes());
                    counter += 1u64;
                    println!("[{:0>8}] {} << \"{}\"", counter, encode(&digest), item);
                    if !set.insert(digest) {
                        eprintln!("Colission has been detected!");
                        eprintln!("The value that caused the colission is: \"{}\"", item);
                        break;
                    }
                }
            }
            Err(_) => {
                eprintln!("Error reading next line!");
                break;
            }
        }
    }

    println!("All items inserted.");
    println!("Total number of unique items is: {}", set.len());
}

criterion_group!(benches, bench_stress);
criterion_main!(benches);
