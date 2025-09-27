// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute};

#[cfg(feature = "logging")]
use simple_logger::SimpleLogger;

fn main() {
    // Initialize the logging sub-system
    #[cfg(feature = "logging")]
    SimpleLogger::new().init().unwrap();

    // Compute digest using the “sone-shot” function
    let digest = compute::<DEFAULT_DIGEST_SIZE>(b"The quick brown fox jumps over the lazy dog");

    // Print result
    println!("{:02X?}", &digest)
}
