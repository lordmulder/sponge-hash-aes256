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
        let mut hash = SpongeHash256::new();
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
        let mut hash = SpongeHash256::new();
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
        let mut hash = SpongeHash256::new();
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
        let mut hash = SpongeHash256::new();
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

#[test]
pub fn test_case_1() {
    do_test(&hex!("1640eed024950214ee2378fa0e0ec14ba84ee84a231d4991634c6f698bd5ee79"), "")
}

#[test]
pub fn test_case_2() {
    do_test(
        &hex!("52fa77818e480fa57a05f4b0eb939204c292b074ab44002624ed02f159bfecf3"),
        "abc",
    )
}

#[test]
pub fn test_case_3() {
    do_test(
        &hex!("6963ccbaf468003edf4ff2aed285e87245f22623687c8d9bdfcca78a2e45389c"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    )
}

#[test]
pub fn test_case_4() {
    do_test(
        &hex!("e5d782b4faee1d7e68ef22de4d676790e67ab99f61b850cfaa89f8926425fa9b"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    )
}

#[test]
pub fn test_case_5() {
    do_test_n(
        &hex!("095ea698657c59cb1ea408a603f5a74a7b56cad391cef4f818ed306e0a02060c"),
        1000000usize,
        "a",
    )
}

#[test]
pub fn test_case_6() {
    do_test_n(
        &hex!("0ffb1ef98ef5a8fe5f85c42f12ef1b58ce4b7e7911043ebadb84d71fc2cec7b8"),
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    )
}
