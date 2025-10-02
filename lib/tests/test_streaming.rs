// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

include!("include/prelude.rs");

use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, DEFAULT_PERMUTE_ROUNDS, SpongeHash256};

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn create_instance(info: Option<&str>) -> SpongeHash256<DEFAULT_PERMUTE_ROUNDS> {
    if let Some(info) = info { SpongeHash256::with_info(info) } else { SpongeHash256::default() }
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

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

include!("include/common.rs");

#[test]
#[ignore]
pub fn test_case_6a() {
    do_test_n(
        &hex!("8a399b4da51ae3a5fafedc98bd04b5886a748e04f22a723d674db2c0e89496c5"),
        None,
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    );
}

#[test]
#[ignore]
pub fn test_case_6b() {
    do_test_n(
        &hex!("b1e723d59ff8d638bdab629db1d1aa7f56e7a024788e483930b5fd61bd23d1e2"),
        Some("thingamajig"),
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    );
}

#[test]
pub fn test_case_7a() {
    do_test_r::<97usize>(
        &hex!("34844d8d1128e830714e6f6d01bb3c48c7b9cd5d68968c886d5274e94ef6ade2"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_7b() {
    do_test_r::<997usize>(
        &hex!("a6934d0662f4130ae5ade5099ea8289253d3b331b31e5d9130d38c76ef016c1e"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_7c() {
    do_test_r::<9973usize>(
        &hex!("b778c95305e2b8d20ab22662e0fe777e38839e10d98b84daa324e8893a77ee4a"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}
