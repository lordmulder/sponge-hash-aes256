// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crate::crypto::{BLOCK_SIZE, aes256_encrypt};
use zeroize::Zeroize;

/// Default digest size, in bytes
pub const DEFAULT_DIGEST_SIZE: usize = 2usize * BLOCK_SIZE;

// ---------------------------------------------------------------------------
// Digest size validator
// ---------------------------------------------------------------------------

/// Validates that the digest size, in bytes, is within the allowed range.
struct ValidDigestSize<const N: usize>;

impl<const N: usize> ValidDigestSize<N> {
    const OK: () = assert!((N > 0) && (N <= 64), "Digest size must be in the [0..=64] range!");
}

// ---------------------------------------------------------------------------
// Streaming API
// ---------------------------------------------------------------------------

/// # SpongeHash-AES256
///
/// The **`SpongeHash256`** struct encapsulates the state for a “streaming” (incremental) SpongeHash-AES256 computation.
pub struct SpongeHash256 {
    state0: [u8; BLOCK_SIZE],
    state1: [u8; BLOCK_SIZE],
    state2: [u8; BLOCK_SIZE],
    offset: usize,
}

impl SpongeHash256 {
    /// Creates a new SpongeHash-AES256 instance and initializes the hash computation.
    pub const fn new() -> Self {
        Self {
            state0: [0x00u8; BLOCK_SIZE],
            state1: [0x36u8; BLOCK_SIZE],
            state2: [0x5Cu8; BLOCK_SIZE],
            offset: 0usize,
        }
    }

    /// Processes the next message chunk, as given by the slice referred to by `message_chunk`.
    ///
    /// The internal state of the hash computation is updated by this function.
    pub fn update(&mut self, message_chunk: &[u8]) -> &Self {
        for byte in message_chunk {
            self.state0[self.offset] ^= byte;
            self.offset += 1usize;
            if self.offset >= BLOCK_SIZE {
                self.iterate();
                self.offset = 0usize;
            }
        }

        self
    }

    /// Concludes the hash computation and returns the final digest.
    ///
    /// The hash value (digest) of the concatenation of all processed message chunks is returned as an new array of size `N`.
    ///
    /// **Note:** The digest size `N`, in bytes, shall be in the 1 to 64 (inclusive) range.
    pub fn digest<const N: usize>(self) -> [u8; N] {
        let () = ValidDigestSize::<N>::OK;
        let mut digest = [0u8; N];
        self.digest_to_slice(&mut digest);
        digest
    }

    /// Concludes the hash computation and returns the final digest.
    ///
    /// The hash value (digest) of the concatenation of all processed message chunks is written into the slice of size `N` referred to by `digest`.
    ///
    /// **Note:** The digest size `N`, in bytes, shall be in the 1 to 64 (inclusive) range.
    pub fn digest_to_slice<const N: usize>(mut self, digest: &mut [u8; N]) {
        let () = ValidDigestSize::<N>::OK;
        assert!(self.offset < BLOCK_SIZE);

        let padding = (BLOCK_SIZE - self.offset) as u8;
        while self.offset < BLOCK_SIZE {
            self.state0[self.offset] ^= padding;
            self.offset += 1usize;
        }

        let mut pos = 0usize;
        while pos < N {
            let copy_len = BLOCK_SIZE.min(N - pos);
            self.iterate();
            digest[pos..(pos + copy_len)].copy_from_slice(&self.state0[..copy_len]);
            pos += copy_len;
        }
    }

    fn iterate(&mut self) {
        let mut temp0 = [0u8; BLOCK_SIZE];
        let mut temp1 = [0u8; BLOCK_SIZE];
        let mut temp2 = [0u8; BLOCK_SIZE];

        aes256_encrypt(&mut temp0, &self.state0, &self.state1, &self.state2);
        aes256_encrypt(&mut temp1, &self.state1, &self.state2, &self.state0);
        aes256_encrypt(&mut temp2, &self.state2, &self.state0, &self.state1);

        for pos in 0..BLOCK_SIZE {
            self.state0[pos] ^= temp0[pos];
            self.state1[pos] ^= temp1[pos];
            self.state2[pos] ^= temp2[pos];
        }

        temp0.zeroize();
        temp1.zeroize();
        temp2.zeroize();
    }
}

impl Default for SpongeHash256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SpongeHash256 {
    fn drop(&mut self) {
        self.state0.zeroize();
        self.state1.zeroize();
        self.state2.zeroize();
    }
}

// ---------------------------------------------------------------------------
// One-Shot API
// ---------------------------------------------------------------------------

/// Convenience function for “sone-shot” SpongeHash-AES256 computation.
///
/// The hash value (digest) of the given `message` is returned as an new array of size `N`.
///
/// **Note:** The digest size `N`, in bytes, shall be in the 1 to 64 (inclusive) range.
pub fn compute<const N: usize>(message: &[u8]) -> [u8; N] {
    let mut state = SpongeHash256::new();
    state.update(message);
    state.digest()
}

/// Convenience function for “sone-shot” SpongeHash-AES256 computation.
///
/// The hash value (digest) of the given `message` is written into the slice of size `N` referred to by `digest`.
///
/// **Note:** The digest size `N`, in bytes, shall be in the 1 to 64 (inclusive) range.
pub fn compute_to_slice<const N: usize>(digest: &mut [u8; N], message: &[u8]) {
    let mut state = SpongeHash256::new();
    state.update(message);
    state.digest_to_slice(digest);
}
