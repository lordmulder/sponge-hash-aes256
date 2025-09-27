// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crate::utilities::{BLOCK_SIZE, aes256_encrypt, xor_arrays};
use zeroize::Zeroize;

#[cfg(feature = "logging")]
use log;

/// Default digest size, in bytes
///
/// The default digest size is currently defined as 32 bytes, i.e., 256 bits.
pub const DEFAULT_DIGEST_SIZE: usize = 2usize * BLOCK_SIZE;

/// Default number of permutation rounds to be performed
///
/// The default number of permutation rounds is currently defined as 3.
pub const DEFAULT_PERMUTE_ROUNDS: usize = 3usize;

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

#[cfg(feature = "logging")]
macro_rules! log {
    ($self:tt, $arg:tt) => {
        log::trace!(
            "SpongeHash256@{:p}: {} --> {:02x?} {:02x?} {:02x?}",
            &$self,
            $arg,
            &$self.state0,
            &$self.state1,
            &$self.state2
        );
    };
}

#[cfg(not(feature = "logging"))]
macro_rules! log {
    ($self:tt, $arg:tt) => {};
}

// ---------------------------------------------------------------------------
// Digest size validator
// ---------------------------------------------------------------------------

/// Validates that the digest size, in bytes, is within the allowed range.
struct ValidDigestSize<const N: usize>;

impl<const N: usize> ValidDigestSize<N> {
    const OK: () = assert!((N > 0) && (N <= 2048), "Digest size must be in the [0..=2048] range!");
}

// ---------------------------------------------------------------------------
// Streaming API
// ---------------------------------------------------------------------------

/// This struct encapsulates the state for a “streaming” (incremental) SpongeHash-AES256 computation.
///
/// ### Usage Example
///
/// The **`SpongeHash256`** struct is used as follows:
///
/// ```rust
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};
///
/// fn main() {
///     // Create new hash instance
///     let mut hash = SpongeHash256::new();
///
///     // Process message
///     hash.update(b"The quick brown fox jumps over the lazy dog");
///
///     // Retrieve the final digest
///     let digest = hash.digest::<DEFAULT_DIGEST_SIZE>();
///
///     // Print result
///     println!("{:02X?}", &digest);
/// }
/// ```
///
/// &nbsp;
///
/// <div class="warning">
///
/// The [`compute()`] and [`compute_to_slice()`] convenience functions may be used as an alternative to working with the `SpongeHash256` struct directly. This is especially useful, if *all* data to be hashed is available at once.
///
/// </div>
pub struct SpongeHash256 {
    state0: [u8; BLOCK_SIZE],
    state1: [u8; BLOCK_SIZE],
    state2: [u8; BLOCK_SIZE],
    rounds: usize,
    offset: usize,
}

impl SpongeHash256 {
    /// Creates a new SpongeHash-AES256 instance and initializes the hash computation.
    ///
    /// This function creates a hash instance that uses [`DEFAULT_PERMUTE_ROUNDS`] permutation rounds.
    pub fn new() -> Self {
        Self::with_rounds(DEFAULT_PERMUTE_ROUNDS)
    }

    /// Creates a new SpongeHash-AES256 instance and initializes the hash computation.
    ///
    /// This function creates a hash instance that uses `rounds` permutation rounds, which must be a *positive* value. A greater value slows down the hash calculation, which helps to increase the security in some usage scenarios, e.g., password hashing.
    pub fn with_rounds(rounds: usize) -> Self {
        if rounds < 1usize {
            panic!("Number of permutation rounds must be a positive value!")
        }

        Self {
            state0: [0x00u8; BLOCK_SIZE],
            state1: [0x36u8; BLOCK_SIZE],
            state2: [0x5Cu8; BLOCK_SIZE],
            rounds,
            offset: 0usize,
        }
    }

    /// Processes the next message chunk, as given by the slice referenced by `message_chunk`.
    ///
    /// The internal state of the hash computation is updated by this function.
    pub fn update(&mut self, message_chunk: &[u8]) {
        log!(self, "update::enter");

        for byte in message_chunk {
            self.state0[self.offset] ^= byte;
            self.offset += 1usize;

            if self.offset >= BLOCK_SIZE {
                self.permute();
                self.offset = 0usize;
            }
        }

        log!(self, "update::leave");
    }

    /// Concludes the hash computation and returns the final digest.
    ///
    /// The hash value (digest) of the concatenation of all processed message chunks is returned as an new array of size `N`.
    ///
    /// **Note:** The digest size `N`, in bytes, shall be in the 1 to 2048 (inclusive) range &#x1F6A8;
    pub fn digest<const N: usize>(self) -> [u8; N] {
        let () = ValidDigestSize::<N>::OK;
        let mut digest = [0u8; N];
        self.digest_to_slice(&mut digest);
        digest
    }

    /// Concludes the hash computation and returns the final digest.
    ///
    /// The hash value (digest) of the concatenation of all processed message chunks is written into the slice referenced by `digest`.
    ///
    /// **Note:** The digest size `N`, in bytes, shall be in the 1 to 2048 (inclusive) range &#x1F6A8;
    pub fn digest_to_slice<const N: usize>(mut self, digest_out: &mut [u8; N]) {
        let () = ValidDigestSize::<N>::OK;
        assert!(self.offset < BLOCK_SIZE);

        log!(self, "digest::enter");

        let padding = (BLOCK_SIZE - self.offset) as u8;
        while self.offset < BLOCK_SIZE {
            self.state0[self.offset] ^= padding;
            self.offset += 1usize;
        }

        let mut pos = 0usize;
        while pos < N {
            let copy_len = BLOCK_SIZE.min(N - pos);
            self.permute();
            digest_out[pos..(pos + copy_len)].copy_from_slice(&self.state0[..copy_len]);
            pos += copy_len;
        }

        log!(self, "digest::leave");
    }

    fn permute(&mut self) {
        log!(self, "permfn::enter");

        let mut temp0 = [0u8; BLOCK_SIZE];
        let mut temp1 = [0u8; BLOCK_SIZE];
        let mut temp2 = [0u8; BLOCK_SIZE];

        for _ in 0..self.rounds {
            aes256_encrypt(&mut temp0, &self.state0, &self.state1, &self.state2);
            aes256_encrypt(&mut temp1, &self.state1, &self.state2, &self.state0);
            aes256_encrypt(&mut temp2, &self.state2, &self.state0, &self.state1);

            xor_arrays(&mut self.state0, &temp0);
            xor_arrays(&mut self.state1, &temp1);
            xor_arrays(&mut self.state2, &temp2);
        }

        temp0.zeroize();
        temp1.zeroize();
        temp2.zeroize();

        log!(self, "permfn::leave");
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

/// Convenience function for “one-shot” SpongeHash-AES256 computation.
///
/// The hash value (digest) of the given `message` is returned as an new array of size `N`.
///
/// This function uses [`DEFAULT_PERMUTE_ROUNDS`] permutation rounds.
///
/// **Note:** The digest size `N`, in bytes, shall be in the 1 to 2048 (inclusive) range &#x1F6A8;
///
/// ### Usage Example
///
/// The **`compute()`** function is used as follows:
///
/// ```rust
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute};
///
/// fn main() {
///     // Compute digest using the “one-shot” function
///     let digest = compute::<DEFAULT_DIGEST_SIZE>(b"The quick brown fox jumps over the lazy dog");
///
///     // Print result
///     println!("{:02X?}", &digest);
/// }
/// ```
///
/// &nbsp;
///
/// <div class="warning">
///
/// Applications that need to process *large* messages are recommended to use the [streaming API](SpongeHash256), which does **not** require *all* message data to be held in memory at once and which allows for an *incremental* hash computation.
///
/// </div>
pub fn compute<const N: usize>(message: &[u8]) -> [u8; N] {
    let mut state = SpongeHash256::new();
    state.update(message);
    state.digest()
}

/// Convenience function for “one-shot” SpongeHash-AES256 computation.
///
/// The hash value (digest) of the given `message` is written into the slice of size `N` referenced by `digest`.
///
/// This function uses [`DEFAULT_PERMUTE_ROUNDS`] permutation rounds.
///
/// **Note:** The digest size `N`, in bytes, shall be in the 1 to 2048 (inclusive) range &#x1F6A8;
///
/// ### Usage Example
///
/// The **`compute()`** function is used as follows:
///
/// ```rust
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute_to_slice};
///
/// fn main() {
///     // Compute digest using the “one-shot” function
///     let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
///     compute_to_slice(&mut digest, b"The quick brown fox jumps over the lazy dog");
///
///     // Print result
///     println!("{:02X?}", &digest);
/// }
/// ```
///
/// &nbsp;
///
/// <div class="warning">
///
/// Applications that need to process *large* messages are recommended to use the [streaming API](SpongeHash256), which does **not** require *all* message data to be held in memory at once and which allows for an *incremental* hash computation.
///
/// </div>
pub fn compute_to_slice<const N: usize>(digest_out: &mut [u8; N], message: &[u8]) {
    let mut state = SpongeHash256::new();
    state.update(message);
    state.digest_to_slice(digest_out);
}
