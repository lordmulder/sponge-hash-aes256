// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use aes::Aes256;
use cipher::{BlockEncrypt, KeyInit};
use core::slice;
use generic_array::GenericArray;
use zeroize::Zeroize;

pub const BLOCK_SIZE: usize = 16usize;
pub const KEY_SIZE: usize = 2usize * BLOCK_SIZE;

// ---------------------------------------------------------------------------
// Alignment checks
// ---------------------------------------------------------------------------

macro_rules! check_aligned {
    ($ptr:expr) => {{
        #[cfg(debug_assertions)]
        {
            $ptr.is_aligned()
        }
        #[cfg(not(debug_assertions))]
        {
            true
        }
    }};
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

#[inline]
pub fn aes256_encrypt(dst: &mut [u8; BLOCK_SIZE], src: &[u8; BLOCK_SIZE], key0: &[u8; BLOCK_SIZE], key1: &[u8; BLOCK_SIZE]) {
    let mut full_key = [0u8; KEY_SIZE];
    full_key[..BLOCK_SIZE].copy_from_slice(key0);
    full_key[BLOCK_SIZE..].copy_from_slice(key1);

    let cipher = Aes256::new(GenericArray::from_slice(&full_key).as_ref());
    full_key.zeroize();

    dst.copy_from_slice(src);
    cipher.encrypt_block(GenericArray::from_mut_slice(dst).as_mut());
}

#[allow(clippy::manual_is_multiple_of)]
#[inline]
pub fn xor_arrays(dst: &mut [u8; BLOCK_SIZE], src: &[u8; BLOCK_SIZE]) {
    const WORDS: usize = BLOCK_SIZE / size_of::<usize>();
    const _: () = assert!(BLOCK_SIZE % size_of::<usize>() == 0usize);

    let (ptr_dst, ptr_src) = (dst.as_ptr() as *mut usize, src.as_ptr() as *const usize);

    if check_aligned!(ptr_src) && check_aligned!(ptr_dst) {
        unsafe {
            let dst_usize = slice::from_raw_parts_mut(ptr_dst, WORDS);
            let src_usize = slice::from_raw_parts(ptr_src, WORDS);
            for (dst_value, src_value) in dst_usize.iter_mut().zip(src_usize.iter()) {
                *dst_value ^= *src_value;
            }
        }
    } else {
        for (dst_value, src_value) in dst.iter_mut().zip(src.iter()) {
            *dst_value ^= *src_value;
        }
    }
}

/// Returns the version of the library as a string
pub const fn version() -> &'static str {
    static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    PKG_VERSION
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    type BlockType = [u8; super::BLOCK_SIZE];

    mod aes256_encrypt {
        use super::{super::*, BlockType};
        use hex_literal::hex;

        const KEY_0: BlockType = hex!("603deb1015ca71be2b73aef0857d7781");
        const KEY_1: BlockType = hex!("1f352c073b6108d72d9810a30914dff4");

        fn do_aes256_ecb(input: &BlockType, expected: &BlockType, key0: &BlockType, key1: &BlockType) {
            let mut output = [0u8; BLOCK_SIZE];
            aes256_encrypt(&mut output, input, key0, key1);
            assert_eq!(&output, expected);
        }

        #[test]
        fn test_aes256_ecb_1a() {
            do_aes256_ecb(&hex!("6bc1bee22e409f96e93d7e117393172a"), &hex!("f3eed1bdb5d2a03c064b5a7e3db181f8"), &KEY_0, &KEY_1);
        }

        #[test]
        fn test_aes256_ecb_1b() {
            do_aes256_ecb(&hex!("6bc1bee22e409f96e93d7e117393172a"), &hex!("5ba1a80938bf65904c5a406f5651b88c"), &KEY_1, &KEY_0);
        }

        #[test]
        fn test_aes256_ecb_2a() {
            do_aes256_ecb(&hex!("ae2d8a571e03ac9c9eb76fac45af8e51"), &hex!("591ccb10d410ed26dc5ba74a31362870"), &KEY_0, &KEY_1);
        }

        #[test]
        fn test_aes256_ecb_2b() {
            do_aes256_ecb(&hex!("ae2d8a571e03ac9c9eb76fac45af8e51"), &hex!("1f38958fe69e4c58d7b0e908000be9b9"), &KEY_1, &KEY_0);
        }

        #[test]
        fn test_aes256_ecb_3a() {
            do_aes256_ecb(&hex!("30c81c46a35ce411e5fbc1191a0a52ef"), &hex!("b6ed21b99ca6f4f9f153e7b1beafed1d"), &KEY_0, &KEY_1);
        }

        #[test]
        fn test_aes256_ecb_3b() {
            do_aes256_ecb(&hex!("30c81c46a35ce411e5fbc1191a0a52ef"), &hex!("139a83bda68fe6438220eaa3aa17e849"), &KEY_1, &KEY_0);
        }

        #[test]
        fn test_aes256_ecb_4a() {
            do_aes256_ecb(&hex!("f69f2445df4f9b17ad2b417be66c3710"), &hex!("23304b7a39f9f3ff067d8d8f9e24ecc7"), &KEY_0, &KEY_1);
        }

        #[test]
        fn test_aes256_ecb_4b() {
            do_aes256_ecb(&hex!("f69f2445df4f9b17ad2b417be66c3710"), &hex!("5b3fbfb893c88a7252f14f5d9a4a0054"), &KEY_1, &KEY_0);
        }
    }

    mod xor_arrays {
        use super::{super::*, BlockType};
        use hex_literal::hex;

        fn do_xor_arrays(input0: &BlockType, input1: &BlockType) {
            let mut output_xor = *input0;
            let mut output_ref = *input0;

            xor_arrays(&mut output_xor, input1);
            for index in 0..BLOCK_SIZE {
                output_ref[index] ^= input1[index];
            }

            assert_eq!(&output_xor, &output_ref);
        }

        #[test]
        fn test_xor_arrays_1() {
            do_xor_arrays(&hex!("75863721fe83cf3d6f0500df428126ae"), &hex!("cc39d4653cce685b8de3398eccfe9c48"));
        }

        #[test]
        fn test_xor_arrays_2() {
            do_xor_arrays(&hex!("2381643e0214c832064a0e8fd074055d"), &hex!("ab290a75923b190ed775841e4cca9e25"));
        }

        #[test]
        fn test_xor_arrays_3() {
            do_xor_arrays(&hex!("62f828dce94781e2d31d9ffa786df6e4"), &hex!("ca6bb37d92d3f8a997d561d9e9d7030e"));
        }

        #[test]
        fn test_xor_arrays_4() {
            do_xor_arrays(&hex!("710180b32b5a982ee21d8e76d287e509"), &hex!("389b742402576214410c0633722c593a"));
        }
    }
}
