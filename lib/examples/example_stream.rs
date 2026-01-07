// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use hex::encode_to_slice;
use sponge_hash_aes256::{SpongeHash256, DEFAULT_DIGEST_SIZE};
use std::str::from_utf8;

#[cfg(feature = "tracing")]
use simple_logger::SimpleLogger;

fn main() {
    // Initialize the logging sub-system
    #[cfg(feature = "tracing")]
    SimpleLogger::new().init().unwrap();

    // Create new hash instance
    let mut hash = SpongeHash256::default();

    // Process message
    hash.update(b"The quick brown fox jumps over the lazy dog");

    // Retrieve the final digest
    let digest = hash.digest::<DEFAULT_DIGEST_SIZE>();

    // Encode to hex
    let mut hex_buffer = [0u8; 2usize * DEFAULT_DIGEST_SIZE];
    encode_to_slice(digest, &mut hex_buffer).unwrap();

    // Print the digest (hex format)
    println!("0x{}", from_utf8(&hex_buffer).unwrap());
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example_stream() {
        super::main();
    }
}
