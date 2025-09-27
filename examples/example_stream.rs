// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};

#[cfg(feature = "logging")]
use simple_logger::SimpleLogger;

fn main() {
    // Initialize the logging sub-system
    #[cfg(feature = "logging")]
    SimpleLogger::new().init().unwrap();

    // Create new hash instance
    let mut hash = SpongeHash256::new();

    // Process message
    hash.update(b"The quick brown fox jumps over the lazy dog");

    // Retrieve the final digest
    let digest = hash.digest::<DEFAULT_DIGEST_SIZE>();

    // Print result
    println!("{:02X?}", &digest);
}
