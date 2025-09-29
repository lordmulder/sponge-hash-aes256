// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use aes::Aes256;
use cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};
use semver::Version;
use zeroize::Zeroize;

pub const BLOCK_SIZE: usize = 16usize;
pub const KEY_SIZE: usize = 2usize * BLOCK_SIZE;

#[inline(always)]
pub fn aes256_encrypt(dst: &mut [u8; BLOCK_SIZE], src: &[u8; BLOCK_SIZE], key0: &[u8; BLOCK_SIZE], key1: &[u8; BLOCK_SIZE]) {
    let mut full_key = [0u8; KEY_SIZE];
    full_key[..BLOCK_SIZE].copy_from_slice(key0);
    full_key[BLOCK_SIZE..].copy_from_slice(key1);

    let cipher = Aes256::new(GenericArray::from_slice(&full_key));
    full_key.zeroize();

    dst.copy_from_slice(src);
    cipher.encrypt_block(GenericArray::from_mut_slice(dst));
}

#[inline(always)]
pub fn xor_arrays<const N: usize>(dst: &mut [u8; N], src: &[u8; N]) {
    debug_assert_eq!(N % size_of::<u64>(), 0usize);
    let word_count: usize = N / size_of::<u64>();

    unsafe {
        let dst = core::slice::from_raw_parts_mut(dst.as_ptr() as *mut u64, word_count);
        let src = core::slice::from_raw_parts(src.as_ptr() as *const u64, word_count);
        for index in 0..word_count {
            dst[index] ^= src[index];
        }
    }
}

/// Returns the library version, as a [Version] struct
pub fn version() -> Version {
    const VERSION_STRING: &str = env!("CARGO_PKG_VERSION");
    Version::parse(VERSION_STRING).unwrap_or_else(|_| Version::new(0u64, 0u64, 0u64))
}
