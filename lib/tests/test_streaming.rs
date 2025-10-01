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
pub fn test_case_6a() {
    do_test_n(
        &hex!("8a399b4da51ae3a5fafedc98bd04b5886a748e04f22a723d674db2c0e89496c5"),
        None,
        16777216usize,
        "aabcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno",
    );
}

#[test]
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
    do_test_r::<251usize>(
        &hex!("3c616508376e0c98d6e1f896d74ffde4b5e9c7e1fea1d73d0bac3141dc695326"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_7b() {
    do_test_r::<65521usize>(
        &hex!("afb916bd22afee21b9e838f23679e09ce79642880b5f18790e71324e183b6d5b"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_7c() {
    do_test_r::<16777213usize>(
        &hex!("fe218c3c203c1b1d5f95b1da43ab84960eb8e616660648e2813145ee525bd5c8"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}
