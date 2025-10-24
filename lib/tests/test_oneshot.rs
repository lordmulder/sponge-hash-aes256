// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

include!("include/prelude.rs");

use sponge_hash_aes256::{compute, compute_to_slice, DEFAULT_DIGEST_SIZE};

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn do_test(expected: &[u8; DEFAULT_DIGEST_SIZE], info: Option<&str>, message: &str) {
    // compute()
    {
        let digest = compute(info, message.as_bytes());
        assert_digest_eq(&digest, expected);
    }

    // compute_to_slice()
    {
        let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
        compute_to_slice(&mut digest, info, message.as_bytes());
        assert_digest_eq(&digest, expected);
    }
}

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

include!("include/common.rs");
