// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use sponge_hash_aes256::{SpongeHash256, DEFAULT_PERMUTE_ROUNDS};
use std::hint::black_box;

#[should_panic(expected = "Info length exceeds the allowable maximum!")]
#[test]
pub fn test_invalid_info_len() {
    black_box(SpongeHash256::<DEFAULT_PERMUTE_ROUNDS>::with_info(str::from_utf8(&[0x61u8; 256usize]).unwrap()));
}
