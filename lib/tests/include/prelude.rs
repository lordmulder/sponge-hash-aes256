// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use hex_literal::hex;
use hex::encode_to_slice;
use std::str::from_utf8;

fn encode<'a>(buffer: &'a mut [u8], data: &[u8]) -> &'a str {
    match encode_to_slice(data, buffer) {
        Ok(_) => from_utf8(buffer).unwrap(),
        Err(_) => Default::default()
    }
}

fn assert_digest_eq<const N: usize>(computed: &[u8; N], expected: &[u8; N]) {
    const BUFF_SIZE: usize = 64usize;

    let mut hex_computed = [0u8; BUFF_SIZE];
    let mut hex_expected = [0u8; BUFF_SIZE];

    assert!(BUFF_SIZE >= 2usize * N);
    assert_eq!(
        computed, expected,
        "Hash mismatch detected:\nexpected=0x{},\ncomputed=0x{}",
        encode(&mut hex_expected, expected), encode(&mut hex_computed, computed)
    );
}
