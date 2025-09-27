// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex_literal::hex;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute, compute_to_slice};

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn do_test(expected: &[u8; DEFAULT_DIGEST_SIZE], message: &str) {
    // compute()
    {
        let digest = compute(message.as_bytes());
        assert_eq!(
            &digest, expected,
            "Hash mismatch detected: expected={:02x?}, computed={:02x?}",
            expected, &digest
        );
    }

    // compute_to_slice()
    {
        let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
        compute_to_slice(&mut digest, message.as_bytes());
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
