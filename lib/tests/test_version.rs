// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use semver::Version;

#[test]
pub fn test_version() {
    let version = Version::parse(sponge_hash_aes256::PKG_VERSION).expect("Failed to parse version!");
    println!("{:?}", version);
}
