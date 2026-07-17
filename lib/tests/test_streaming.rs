// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

include!("include/utils.rs");

use sponge_hash_aes256::{SpongeHash256, DEFAULT_DIGEST_SIZE, DEFAULT_PERMUTE_ROUNDS};

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn create_instance(info: Option<&str>) -> SpongeHash256<DEFAULT_PERMUTE_ROUNDS> {
    if let Some(info) = info {
        SpongeHash256::with_info(info)
    } else {
        SpongeHash256::default()
    }
}

fn do_test(expected: &[u8; DEFAULT_DIGEST_SIZE], info: Option<&str>, message: &str) {
    // SpongeHash256::digest()
    {
        let mut hash = create_instance(info);
        hash.update(message.as_bytes());
        let digest = hash.digest();
        assert_digest_eq(&digest, expected);
    }

    // SpongeHash256::digest_to_slice()
    {
        let mut hash = create_instance(info);
        hash.update(message.as_bytes());
        let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
        hash.digest_to_slice(&mut digest);
        assert_digest_eq(&digest, expected);
    }
}

fn do_test_n(expected: &[u8; DEFAULT_DIGEST_SIZE], info: Option<&str>, count: usize, message: &str) {
    // SpongeHash256::digest()
    {
        let mut hash = create_instance(info);
        for _ in 0..count {
            hash.update(message.as_bytes());
        }
        let digest = hash.digest();
        assert_digest_eq(&digest, expected);
    }

    // SpongeHash256::digest_to_slice()
    {
        let mut hash = create_instance(info);
        for _ in 0..count {
            hash.update(message.as_bytes());
        }
        let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
        hash.digest_to_slice(&mut digest);
        assert_digest_eq(&digest, expected);
    }
}

fn do_test_r<const R: usize>(expected: &[u8; DEFAULT_DIGEST_SIZE], message: &str) {
    let mut hash = SpongeHash256::<R>::new();
    hash.update(message.as_bytes());
    let digest = hash.digest();
    assert_digest_eq(&digest, expected);
}

fn do_test_c(expected: &[u8; DEFAULT_DIGEST_SIZE], info: Option<&str>, message_1: &str, message_2: &str) {
    let mut hash_1 = create_instance(info);
    hash_1.update(message_1.as_bytes());
    let mut hash_2 = hash_1.clone();
    hash_1.update(message_2.as_bytes());
    hash_2.update(message_2.as_bytes());
    let digest_1 = hash_1.digest();
    let digest_2 = hash_2.digest();
    assert_digest_eq(&digest_1, expected);
    assert_digest_eq(&digest_2, expected);
}

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

include!("include/common.rs");

#[test]
#[ignore]
pub fn test_case_6a() {
    do_test_n(
        &hex!("0319430f76325543f731d2015306c1030fb4c4498e5dca8629ccc62d68ddcc9d"),
        None,
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    );
}

#[test]
#[ignore]
pub fn test_case_6b() {
    do_test_n(
        &hex!("b5a73e268d160dfb5407dcbf40591ab73111b2e0928139b3c8ec8bdf9b132a65"),
        Some("thingamajig"),
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    );
}

#[test]
pub fn test_case_7a() {
    do_test_r::<13usize>(
        &hex!("5320f5bd6c572483d9c484d3022cd9d2b9a072897a66ff1a517d00302da5674b"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_7b() {
    do_test_r::<251usize>(
        &hex!("3340d0e0d5261974273b2ae0b438c876784a8deaf64d38e4e92673036ef124c4"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_7c() {
    do_test_r::<4093usize>(
        &hex!("4aa2cff9859d03abe0e1387c0923f347cc8145b8562e308088cbda36e23c0fbb"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_7d() {
    do_test_r::<65521usize>(
        &hex!("af2281df4ad2a2a989c5f750723754d2a2d823d6bfcc0b91058e629d4eda5f74"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_8a() {
    do_test_c(
        &hex!("8bd72fc391b6ae4ed1fe352be88cf3d8e496ab8283a4131233a9635d8db10855"),
        None,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_8b() {
    do_test_c(
        &hex!("fd9830187ce1b95c674e829c35e603496ec8df0f0a2788f76c1ffd9d804b6f23"),
        Some("thingamajig"),
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}
