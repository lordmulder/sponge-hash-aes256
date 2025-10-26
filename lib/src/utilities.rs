// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use core::ops::{Index, IndexMut};

use aes::Aes256;
use cipher::{BlockEncrypt, KeyInit};
use generic_array::GenericArray;
use zeroize::Zeroize;

pub const BLOCK_SIZE: usize = 16usize;
pub const KEY_SIZE: usize = 2usize * BLOCK_SIZE;

// ---------------------------------------------------------------------------
// Block type
// ---------------------------------------------------------------------------

/// Represents an aligned 128-Bit block
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[repr(align(128))]
pub struct BlockType(pub [u8; BLOCK_SIZE]);

impl BlockType {
    /// Forces *self* to be overwritten completely with zeros, earsing the previous content
    #[inline(always)]
    pub fn zeroize(&mut self) {
        self.0.zeroize();
    }

    /// Computes the bit-wise XOR of `other` and *self*, stores the result "in-place" in *self*
    #[inline(always)]
    pub fn xor_with(&mut self, other: &BlockType) {
        let (dst_ptr, src_ptr) = (self.0.as_mut_ptr() as *mut u128, other.0.as_ptr() as *const u128);
        unsafe {
            *dst_ptr ^= *src_ptr;
        }
    }
}

impl Index<usize> for BlockType {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for BlockType {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Encrypes the 128-bit block `src` with AES-256 and stores the result in `dst`.
///
/// The 128 key bits from `key0` and the 128 key bits from `key1` are concatenated to form a full 256-bit key.
#[inline]
pub fn aes256_encrypt(dst: &mut BlockType, src: &BlockType, key0: &BlockType, key1: &BlockType) {
    let mut full_key = [0u8; KEY_SIZE];
    full_key[..BLOCK_SIZE].copy_from_slice(&key0.0);
    full_key[BLOCK_SIZE..].copy_from_slice(&key1.0);

    let cipher = Aes256::new(GenericArray::from_slice(&full_key).as_ref());
    full_key.zeroize();

    dst.0.copy_from_slice(&src.0);
    cipher.encrypt_block(GenericArray::from_mut_slice(&mut dst.0).as_mut());
}

/// Returns the version of the library as a string
pub const fn version() -> &'static str {
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
    PKG_VERSION
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    mod aes256_encrypt {
        use super::super::*;
        use hex_literal::hex;

        const KEY_0: BlockType = BlockType(hex!("603deb1015ca71be2b73aef0857d7781"));
        const KEY_1: BlockType = BlockType(hex!("1f352c073b6108d72d9810a30914dff4"));

        fn do_aes256_ecb(input: &BlockType, expected: &BlockType, key0: &BlockType, key1: &BlockType) {
            let mut output = BlockType::default();
            aes256_encrypt(&mut output, input, key0, key1);
            assert_eq!(&output, expected);
        }

        #[test]
        fn test_aes256_ecb_1a() {
            do_aes256_ecb(
                &BlockType(hex!("6bc1bee22e409f96e93d7e117393172a")),
                &BlockType(hex!("f3eed1bdb5d2a03c064b5a7e3db181f8")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_1b() {
            do_aes256_ecb(
                &BlockType(hex!("6bc1bee22e409f96e93d7e117393172a")),
                &BlockType(hex!("5ba1a80938bf65904c5a406f5651b88c")),
                &KEY_1,
                &KEY_0,
            );
        }

        #[test]
        fn test_aes256_ecb_2a() {
            do_aes256_ecb(
                &BlockType(hex!("ae2d8a571e03ac9c9eb76fac45af8e51")),
                &BlockType(hex!("591ccb10d410ed26dc5ba74a31362870")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_2b() {
            do_aes256_ecb(
                &BlockType(hex!("ae2d8a571e03ac9c9eb76fac45af8e51")),
                &BlockType(hex!("1f38958fe69e4c58d7b0e908000be9b9")),
                &KEY_1,
                &KEY_0,
            );
        }

        #[test]
        fn test_aes256_ecb_3a() {
            do_aes256_ecb(
                &BlockType(hex!("30c81c46a35ce411e5fbc1191a0a52ef")),
                &BlockType(hex!("b6ed21b99ca6f4f9f153e7b1beafed1d")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_3b() {
            do_aes256_ecb(
                &BlockType(hex!("30c81c46a35ce411e5fbc1191a0a52ef")),
                &BlockType(hex!("139a83bda68fe6438220eaa3aa17e849")),
                &KEY_1,
                &KEY_0,
            );
        }

        #[test]
        fn test_aes256_ecb_4a() {
            do_aes256_ecb(
                &BlockType(hex!("f69f2445df4f9b17ad2b417be66c3710")),
                &BlockType(hex!("23304b7a39f9f3ff067d8d8f9e24ecc7")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_4b() {
            do_aes256_ecb(
                &BlockType(hex!("f69f2445df4f9b17ad2b417be66c3710")),
                &BlockType(hex!("5b3fbfb893c88a7252f14f5d9a4a0054")),
                &KEY_1,
                &KEY_0,
            );
        }
    }

    mod xor_arrays {
        use super::super::*;
        use hex_literal::hex;

        fn do_xor_arrays(input0: &BlockType, input1: &BlockType) {
            let mut output_xor = input0.clone();
            let mut output_ref = input0.clone();

            output_xor.xor_with(input1);
            for index in 0..BLOCK_SIZE {
                output_ref[index] ^= input1[index];
            }

            assert_eq!(&output_xor, &output_ref);
        }

        #[test]
        fn test_xor_arrays_1() {
            do_xor_arrays(
                &BlockType(hex!("75863721fe83cf3d6f0500df428126ae")),
                &BlockType(hex!("cc39d4653cce685b8de3398eccfe9c48")),
            );
        }

        #[test]
        fn test_xor_arrays_2() {
            do_xor_arrays(
                &BlockType(hex!("2381643e0214c832064a0e8fd074055d")),
                &BlockType(hex!("ab290a75923b190ed775841e4cca9e25")),
            );
        }

        #[test]
        fn test_xor_arrays_3() {
            do_xor_arrays(
                &BlockType(hex!("62f828dce94781e2d31d9ffa786df6e4")),
                &BlockType(hex!("ca6bb37d92d3f8a997d561d9e9d7030e")),
            );
        }

        #[test]
        fn test_xor_arrays_4() {
            do_xor_arrays(
                &BlockType(hex!("710180b32b5a982ee21d8e76d287e509")),
                &BlockType(hex!("389b742402576214410c0633722c593a")),
            );
        }
    }
}
