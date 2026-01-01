// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use semver::Version;

static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[test]
pub fn test_version() {
    let version_expected = Version::parse(PKG_VERSION).expect("Failed to parse version!");
    let version_returned = Version::parse(sponge_hash_aes256::version()).expect("Failed to parse version!");
    assert!(version_returned.build.is_empty());
    assert!(version_returned.pre.is_empty());
    assert_eq!(version_returned, version_expected);
}
