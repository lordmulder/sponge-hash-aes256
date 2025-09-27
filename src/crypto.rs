// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use aes::Aes256;
use cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use zeroize::Zeroize;

pub const BLOCK_SIZE: usize = 16usize;
pub const KEY_SIZE: usize = 2usize * BLOCK_SIZE;

pub fn aes256_encrypt(dst: &mut [u8; BLOCK_SIZE], src: &[u8; BLOCK_SIZE], key0: &[u8; BLOCK_SIZE], key1: &[u8; BLOCK_SIZE]) {
    let mut full_key = [0u8; KEY_SIZE];
    full_key[..BLOCK_SIZE].copy_from_slice(key0);
    full_key[BLOCK_SIZE..].copy_from_slice(key1);

    let cipher = Aes256::new(GenericArray::from_slice(&full_key));
    full_key.zeroize();

    dst.copy_from_slice(src);
    cipher.encrypt_block(GenericArray::from_mut_slice(dst));
}
