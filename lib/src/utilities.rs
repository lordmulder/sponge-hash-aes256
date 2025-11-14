// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use aes::{
    cipher::{BlockCipherEncrypt, Key, KeyInit},
    Aes256,
};
use core::{
    mem::MaybeUninit,
    ops::{Index, IndexMut, RangeTo},
    ptr,
};
use wide::u8x16;
use zeroize::Zeroize;

pub const BLOCK_SIZE: usize = 16usize;
pub const ZERO: u8x16 = u8x16::ZERO;

// ---------------------------------------------------------------------------
// Block type
// ---------------------------------------------------------------------------

/// Represents an aligned 128-Bit block
#[derive(Clone, Debug)]
#[repr(align(16))]
pub struct BlockType(u8x16);

impl BlockType {
    /// Create a new block that is initialized entirely from the given `INIT_VALUE`
    #[inline(always)]
    pub const fn new<const INIT_VALUE: u8>() -> Self {
        Self(u8x16::new([INIT_VALUE; BLOCK_SIZE]))
    }

    /// Create a new block that is initialized from the given array
    #[allow(dead_code)]
    #[inline(always)]
    pub const fn from_array(value: [u8; BLOCK_SIZE]) -> Self {
        Self(u8x16::new(value))
    }

    /// Create a new block that is initialized to "zero" bytes
    #[inline(always)]
    pub const fn zero() -> Self {
        unsafe { Self(MaybeUninit::zeroed().assume_init()) }
    }

    /// Create a new block that is *not* initialized to any particular state
    #[allow(invalid_value)]
    #[allow(clippy::uninit_assumed_init)]
    #[inline(always)]
    pub const fn uninit() -> Self {
        unsafe { Self(MaybeUninit::uninit().assume_init()) }
    }

    /// Computes the bit-wise XOR of `other` and *self*, stores the result "in-place" in *self*
    #[inline(always)]
    pub fn xor_with(&mut self, other: &Self) {
        self.0 ^= other.0;
    }

    /// Get a `&[u8; BLOCK_SIZE]` reference to the contained data
    #[allow(dead_code)]
    #[inline(always)]
    fn as_array(&self) -> &[u8; BLOCK_SIZE] {
        self.0.as_array()
    }

    /// Get a `&mut [u8; BLOCK_SIZE]` reference to the contained data
    #[inline(always)]
    fn as_mut_array(&mut self) -> &mut [u8; BLOCK_SIZE] {
        self.0.as_mut_array()
    }

    /// Get a "raw" `*const u8` pointer to the contained data
    #[inline(always)]
    fn as_ptr(&self) -> *const [u8; BLOCK_SIZE] {
        self.0.as_array().as_ptr() as *const [u8; BLOCK_SIZE]
    }
}

impl Index<usize> for BlockType {
    type Output = u8;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0.as_array()[index]
    }
}

impl IndexMut<usize> for BlockType {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0.as_mut_array()[index]
    }
}

impl Index<RangeTo<usize>> for BlockType {
    type Output = [u8];

    #[inline(always)]
    fn index(&self, range: RangeTo<usize>) -> &Self::Output {
        &self.0.as_array()[range]
    }
}

impl PartialEq for BlockType {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 ^ other.0 == ZERO
    }
}

impl Drop for BlockType {
    #[inline(always)]
    fn drop(&mut self) {
        self.0.as_mut_array().zeroize();
    }
}

// ---------------------------------------------------------------------------
// Key type
// ---------------------------------------------------------------------------

/// Represents an aligned 256-Bit key
#[repr(align(32))]
pub struct KeyType(Key<Aes256>);

impl KeyType {
    /// Concatenate the two 128-bit blocks `key0` and `key1` to from a full 256-bit key
    #[allow(invalid_value)]
    #[allow(clippy::uninit_assumed_init)]
    #[inline(always)]
    pub const fn uninit() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }

    /// Concatenate the two 128-bit blocks `key0` and `key1` to from a full 256-bit key
    #[inline(always)]
    pub fn concat(&mut self, key0: &BlockType, key1: &BlockType) -> &Key<Aes256> {
        unsafe {
            let write_ptr = self.0.as_mut_ptr() as *mut [u8; BLOCK_SIZE];
            ptr::copy_nonoverlapping(key0.as_ptr(), write_ptr, 1usize);
            ptr::copy_nonoverlapping(key1.as_ptr(), write_ptr.add(1usize), 1usize);
        }
        &self.0
    }
}

impl Drop for KeyType {
    #[inline(always)]
    fn drop(&mut self) {
        self.0.as_mut_slice().zeroize();
    }
}

// ---------------------------------------------------------------------------
// AES-256 Utility
// ---------------------------------------------------------------------------

/// Handles encryption with the AES-256 block cipher
pub struct Aes256Crypto {
    key: KeyType,
}

impl Aes256Crypto {
    /// Encrypes the 128-bit block `src` with AES-256 and stores the result in `dst`.
    ///
    /// The 128 key bits from `key0` and the 128 key bits from `key1` are concatenated to form a full 256-bit key.
    #[inline]
    pub fn encrypt(&mut self, dst: &mut BlockType, src: &BlockType, key0: &BlockType, key1: &BlockType) {
        let cipher = Aes256::new(self.key.concat(key0, key1));
        cipher.encrypt_block_b2b(src.as_array().into(), dst.as_mut_array().into());
    }
}

impl Default for Aes256Crypto {
    /// Creates a new `Aes256Processor` instance
    #[inline]
    fn default() -> Self {
        Self { key: KeyType::uninit() }
    }
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

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

        const KEY_0: BlockType = BlockType::from_array(hex!("603deb1015ca71be2b73aef0857d7781"));
        const KEY_1: BlockType = BlockType::from_array(hex!("1f352c073b6108d72d9810a30914dff4"));

        fn do_aes256_ecb(input: &BlockType, expected: &BlockType, key0: &BlockType, key1: &BlockType) {
            let mut output = BlockType::zero();
            Aes256Crypto::default().encrypt(&mut output, input, key0, key1);
            assert_eq!(&output, expected);
        }

        #[test]
        fn test_aes256_ecb_1a() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("6bc1bee22e409f96e93d7e117393172a")),
                &BlockType::from_array(hex!("f3eed1bdb5d2a03c064b5a7e3db181f8")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_1b() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("6bc1bee22e409f96e93d7e117393172a")),
                &BlockType::from_array(hex!("5ba1a80938bf65904c5a406f5651b88c")),
                &KEY_1,
                &KEY_0,
            );
        }

        #[test]
        fn test_aes256_ecb_2a() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("ae2d8a571e03ac9c9eb76fac45af8e51")),
                &BlockType::from_array(hex!("591ccb10d410ed26dc5ba74a31362870")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_2b() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("ae2d8a571e03ac9c9eb76fac45af8e51")),
                &BlockType::from_array(hex!("1f38958fe69e4c58d7b0e908000be9b9")),
                &KEY_1,
                &KEY_0,
            );
        }

        #[test]
        fn test_aes256_ecb_3a() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("30c81c46a35ce411e5fbc1191a0a52ef")),
                &BlockType::from_array(hex!("b6ed21b99ca6f4f9f153e7b1beafed1d")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_3b() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("30c81c46a35ce411e5fbc1191a0a52ef")),
                &BlockType::from_array(hex!("139a83bda68fe6438220eaa3aa17e849")),
                &KEY_1,
                &KEY_0,
            );
        }

        #[test]
        fn test_aes256_ecb_4a() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("f69f2445df4f9b17ad2b417be66c3710")),
                &BlockType::from_array(hex!("23304b7a39f9f3ff067d8d8f9e24ecc7")),
                &KEY_0,
                &KEY_1,
            );
        }

        #[test]
        fn test_aes256_ecb_4b() {
            do_aes256_ecb(
                &BlockType::from_array(hex!("f69f2445df4f9b17ad2b417be66c3710")),
                &BlockType::from_array(hex!("5b3fbfb893c88a7252f14f5d9a4a0054")),
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

            for (dst, src) in output_ref.as_mut_array().iter_mut().zip(input1.as_array().iter()) {
                *dst ^= src;
            }

            assert_eq!(&output_xor, &output_ref);
        }

        #[test]
        fn test_xor_arrays_1() {
            do_xor_arrays(
                &BlockType::from_array(hex!("75863721fe83cf3d6f0500df428126ae")),
                &BlockType::from_array(hex!("cc39d4653cce685b8de3398eccfe9c48")),
            );
        }

        #[test]
        fn test_xor_arrays_2() {
            do_xor_arrays(
                &BlockType::from_array(hex!("2381643e0214c832064a0e8fd074055d")),
                &BlockType::from_array(hex!("ab290a75923b190ed775841e4cca9e25")),
            );
        }

        #[test]
        fn test_xor_arrays_3() {
            do_xor_arrays(
                &BlockType::from_array(hex!("62f828dce94781e2d31d9ffa786df6e4")),
                &BlockType::from_array(hex!("ca6bb37d92d3f8a997d561d9e9d7030e")),
            );
        }

        #[test]
        fn test_xor_arrays_4() {
            do_xor_arrays(
                &BlockType::from_array(hex!("710180b32b5a982ee21d8e76d287e509")),
                &BlockType::from_array(hex!("389b742402576214410c0633722c593a")),
            );
        }
    }
}
