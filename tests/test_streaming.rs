// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex_literal::hex;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn do_test(expected: &[u8; DEFAULT_DIGEST_SIZE], message: &str) {
    // SpongeHash256::digest()
    {
        let mut hash = SpongeHash256::default();
        hash.update(message.as_bytes());
        let digest = hash.digest();
        assert_eq!(
            &digest, expected,
            "Hash mismatch detected: expected={:02x?}, computed={:02x?}",
            expected, &digest
        );
    }

    // SpongeHash256::digest_to_slice()
    {
        let mut hash = SpongeHash256::default();
        hash.update(message.as_bytes());
        let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
        hash.digest_to_slice(&mut digest);
        assert_eq!(
            &digest, expected,
            "Hash mismatch detected: expected={:02x?}, computed={:02x?}",
            expected, &digest
        );
    }
}

fn do_test_n(expected: &[u8; DEFAULT_DIGEST_SIZE], count: usize, message: &str) {
    // SpongeHash256::digest()
    {
        let mut hash = SpongeHash256::default();
        for _ in 0..count {
            hash.update(message.as_bytes());
        }
        let digest = hash.digest();
        assert_eq!(
            &digest, expected,
            "Hash mismatch detected: expected={:02x?}, computed={:02x?}",
            expected, &digest
        );
    }

    // SpongeHash256::digest_to_slice()
    {
        let mut hash = SpongeHash256::default();
        for _ in 0..count {
            hash.update(message.as_bytes());
        }
        let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
        hash.digest_to_slice(&mut digest);
        assert_eq!(
            &digest, expected,
            "Hash mismatch detected: expected={:02x?}, computed={:02x?}",
            expected, &digest
        );
    }
}

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

include!("include/common.rs");

#[test]
pub fn test_case_6() {
    do_test_n(
        &hex!("029d9ba8074b2bdd2b67f261bf354d16a9fbd5c0e13bf302f98f9ca3478044b9"),
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    )
}
