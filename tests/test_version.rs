// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#[test]
pub fn test_version() {
    let version = sponge_hash_aes256::version();
    println!("{:?}", version);
}
