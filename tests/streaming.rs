// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hash::{DEFAULT_DIGEST_SIZE, SpongeHash256};
use hex_literal::hex;

fn do_test(expected: &[u8; DEFAULT_DIGEST_SIZE], message: &str) {
    let mut hash = SpongeHash256::new();
    hash.update(message.as_bytes());
    let digest = hash.digest();
    assert_eq!(
        &digest, expected,
        "Hash mismatch detected: expected={:02x?}, computed={:02x?}",
        expected, &digest
    );
}

fn do_test_n(expected: &[u8; DEFAULT_DIGEST_SIZE], count: usize, message: &str) {
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

#[test]
pub fn test_case_1() {
    do_test(&hex!("8a897a202c82037e19dee41d718cc08ce7c9cd93d57169905cad5b3723cf51a7"), "")
}

#[test]
pub fn test_case_2() {
    do_test(
        &hex!("27de1d44b1de0f39079cefd1a8c5facb295631184b3ac19b6dc8bdcd2452f78c"),
        "abc",
    )
}

#[test]
pub fn test_case_3() {
    do_test(
        &hex!("c2e429656e20ed5dda79fa827725947dd5bf0b468087495bdb4834767dde2dfb"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    )
}

#[test]
pub fn test_case_4() {
    do_test(
        &hex!("4a6e8fefa4ef867f783e195df1d912b2a0f9aa400045ad4b9372ded1000d9ed0"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    )
}

#[test]
pub fn test_case_5() {
    do_test_n(
        &hex!("dccdc991620ef4950fa05e195efbcf1ff75f362dd6a079b07d8b717180f993c2"),
        1000000usize,
        "a",
    )
}

#[test]
pub fn test_case_6() {
    do_test_n(
        &hex!("3f7826e74fce8d042f262744f9845e2cb0c6a7242ea68fcc6a0c220422089bbf"),
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    )
}
