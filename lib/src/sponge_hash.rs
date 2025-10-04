// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use crate::utilities::{BLOCK_SIZE, aes256_encrypt, xor_arrays};
use zeroize::Zeroize;

/// Default digest size, in bytes
///
/// The default digest size is currently defined as **32** bytes, i.e., 256 bits.
pub const DEFAULT_DIGEST_SIZE: usize = 2usize * BLOCK_SIZE;

/// Default number of permutation rounds to be performed
///
/// The default number of permutation rounds is currently defined as **1**.
pub const DEFAULT_PERMUTE_ROUNDS: usize = 1usize;

// ---------------------------------------------------------------------------
// Tracing
// ---------------------------------------------------------------------------

#[cfg(feature = "tracing")]
macro_rules! trace {
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

#[cfg(not(feature = "tracing"))]
macro_rules! trace {
    ($self:tt, $arg:tt) => {};
}

// ---------------------------------------------------------------------------
// Non-zero argument constraint
// ---------------------------------------------------------------------------

/// Validates that the const generic parameter is non-zero
struct NoneZeroArg<const N: usize>;

impl<const N: usize> NoneZeroArg<N> {
    const OK: () = assert!(N > 0, "Const generic argument must be a non-zero value!");
}

// ---------------------------------------------------------------------------
// Streaming API
// ---------------------------------------------------------------------------

/// This struct encapsulates the state for a “streaming” (incremental) SpongeHash-AES256 computation.
///
/// The const generic parameter `R` specifies the number of permutation rounds to be performed, which must be a *positive* value. The default number of permutation rounds is given by [`DEFAULT_PERMUTE_ROUNDS`]. Using a greater value slows down the hash calculation, which helps to increase the security in some usage scenarios, e.g., password hashing.
///
/// ### Usage Example
///
/// The easiest way to use the **`SpongeHash256`** structure is as follows:
///
/// ```rust
/// use hex::encode_to_slice;
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};
///
/// fn main() {
///     // Create new hash instance
///     let mut hash = SpongeHash256::default();
///
///     // Process message
///     hash.update(b"The quick brown fox jumps over the lazy dog");
///
///     // Retrieve the final digest
///     let digest = hash.digest::<DEFAULT_DIGEST_SIZE>();
///
///     // Encode to hex
///     let mut hex_buffer = [0u8; 2usize * DEFAULT_DIGEST_SIZE];
///     encode_to_slice(&digest, &mut hex_buffer).unwrap();
///
///     // Print the digest (hex format)
///     println!("0x{}", str::from_utf8(&hex_buffer).unwrap());
/// }
/// ```
///
/// ### Context information
///
/// Optionally, additional “context” information may be provided via the `info` parameter:
///
/// ```rust
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, SpongeHash256};
///
/// fn main() {
///     // Create new hash instance with “info”
///     let mut hash: SpongeHash256 = SpongeHash256::with_info("my_application");
///
///     /* ... */
/// }
/// ```
///
/// ### Important note
///
/// <div class="warning">
///
/// The [`compute()`] and [`compute_to_slice()`] convenience functions may be used as an alternative to working with the `SpongeHash256` struct directly. This is especially useful, if *all* data to be hashed is available at once.
///
/// </div>
pub struct SpongeHash256<const R: usize = DEFAULT_PERMUTE_ROUNDS> {
    state0: [u8; BLOCK_SIZE],
    state1: [u8; BLOCK_SIZE],
    state2: [u8; BLOCK_SIZE],
    offset: usize,
}

impl<const R: usize> SpongeHash256<R> {
    const BIT_MASK_X: [u8; BLOCK_SIZE] = [0x5Cu8; BLOCK_SIZE];
    const BIT_MASK_Y: [u8; BLOCK_SIZE] = [0x36u8; BLOCK_SIZE];
    const BIT_MASK_Z: [u8; BLOCK_SIZE] = [0x6Au8; BLOCK_SIZE];

    /// Creates a new SpongeHash-AES256 instance and initializes the hash computation.
    ///
    /// **Note:** This function implies an *empty* [`info`](Self::with_info()) string.
    pub fn new() -> Self {
        Self::with_info(Default::default())
    }

    /// Creates a new SpongeHash-AES256 instance and initializes the hash computation with the given `info` string.
    ///
    /// **Note:** The length of the `info` string **must not** exceed a length of 255 characters!
    pub fn with_info(info: &str) -> Self {
        let () = NoneZeroArg::<R>::OK;

        let mut instance =
            Self { state0: [0u8; BLOCK_SIZE], state1: [0u8; BLOCK_SIZE], state2: [0u8; BLOCK_SIZE], offset: 0usize };

        instance.initialize(info.as_bytes());
        instance
    }

    /// Initializes the internal state with the given `info` string
    fn initialize(&mut self, info_data: &[u8]) {
        trace!(self, "initlz::enter");

        match info_data.len().try_into() {
            Ok(length) => {
                self.update(u8::to_be_bytes(length));
                self.update(info_data);
            }
            Err(_) => panic!("Info length exceeds the allowable maximum!"),
        };

        trace!(self, "initlz::leave");
    }

    /// Processes the next chunk of the message, as given by the `chunk` parameter.
    ///
    /// A `chunk` can be of *any* type that implements the [`AsRef<[u8]>`](AsRef<T>) trait, e.g., `&[u8]`, `&str` or `String`.
    ///
    /// The internal state of the hash computation is updated by this function.
    pub fn update<T: AsRef<[u8]>>(&mut self, chunk: T) {
        trace!(self, "update::enter");

        for byte in chunk.as_ref() {
            self.state0[self.offset] ^= byte;
            self.offset += 1usize;

            if self.offset >= BLOCK_SIZE {
                self.permute();
                self.offset = 0usize;
            }
        }

        trace!(self, "update::leave");
    }

    /// Concludes the hash computation and returns the final digest.
    ///
    /// The hash value (digest) of the concatenation of all processed message chunks is returned as an new array of size `N`.
    ///
    /// The returned array is filled completely, generating a hash value (digest) of the appropriate size.
    ///
    /// **Note:** The digest output size `N`, in bytes, must be a *positive* value! &#x1F6A8;
    pub fn digest<const N: usize>(self) -> [u8; N] {
        let () = NoneZeroArg::<N>::OK;
        let mut digest = [0u8; N];
        self.digest_to_slice(&mut digest);
        digest
    }

    /// Concludes the hash computation and returns the final digest.
    ///
    /// The hash value (digest) of the concatenation of all processed message chunks is written into the slice `digest_out`.
    ///
    /// The output slice is filled completely, generating a hash value (digest) of the appropriate size.
    ///
    /// **Note:** The specified digest output size, i.e., `digest_out.len()`, in bytes, must be a *positive* value! &#x1F6A8;
    pub fn digest_to_slice(mut self, digest_out: &mut [u8]) {
        trace!(self, "digest::enter");
        assert!(!digest_out.is_empty(), "Digest output size must be positive!");

        self.state0[self.offset] ^= 0x80u8;
        let mut pos = 0usize;

        self.permute();
        xor_arrays(&mut self.state0, &Self::BIT_MASK_Z);

        while pos < digest_out.len() {
            self.permute();
            let copy_len = BLOCK_SIZE.min(digest_out.len() - pos);
            digest_out[pos..(pos + copy_len)].copy_from_slice(&self.state0[..copy_len]);
            pos += copy_len;
        }

        trace!(self, "digest::leave");
    }

    /// Pseudorandom permutation, based on the AES-256 block cipher
    fn permute(&mut self) {
        trace!(self, "permfn::enter");

        let mut temp0 = [0u8; BLOCK_SIZE];
        let mut temp1 = [0u8; BLOCK_SIZE];
        let mut temp2 = [0u8; BLOCK_SIZE];

        for _ in 0..R {
            aes256_encrypt(&mut temp0, &self.state0, &self.state1, &self.state2);
            aes256_encrypt(&mut temp1, &self.state1, &self.state2, &self.state0);
            aes256_encrypt(&mut temp2, &self.state2, &self.state0, &self.state1);

            xor_arrays(&mut self.state0, &temp0);
            xor_arrays(&mut self.state1, &temp1);
            xor_arrays(&mut self.state2, &temp2);

            xor_arrays(&mut self.state1, &Self::BIT_MASK_X);
            xor_arrays(&mut self.state2, &Self::BIT_MASK_Y);
        }

        temp0.zeroize();
        temp1.zeroize();
        temp2.zeroize();

        trace!(self, "permfn::leave");
    }
}

impl Default for SpongeHash256 {
    fn default() -> Self {
        Self::new()
    }
}

impl<const R: usize> Drop for SpongeHash256<R> {
    fn drop(&mut self) {
        self.state0.zeroize();
        self.state1.zeroize();
        self.state2.zeroize();
    }
}

// ---------------------------------------------------------------------------
// One-Shot API
// ---------------------------------------------------------------------------

/// Convenience function for “one-shot” SpongeHash-AES256 computation
///
/// The hash value (digest) of the given `message` is returned as an new array of type `[u8; N]`.
///
/// A `message` can be of *any* type that implements the [`AsRef<[u8]>`](AsRef<T>) trait, e.g., `&[u8]`, `&str` or `String`.
///
/// Optionally, an additional `info` string may be specified.
///
/// The returned array is filled completely, generating a hash value (digest) of the appropriate size.
///
/// This function uses the default number of permutation rounds, as is given by [`DEFAULT_PERMUTE_ROUNDS`].
///
/// **Note:** The digest output size `N`, in bytes, must be a *positive* value! &#x1F6A8;
///
/// ### Usage Example
///
/// The **`compute()`** function can be used as follows:
///
/// ```rust
/// use hex::encode_to_slice;
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute};
///
/// fn main() {
///     // Compute digest using the “one-shot” function
///     let digest: [u8; DEFAULT_DIGEST_SIZE] = compute(
///         None,
///         b"The quick brown fox jumps over the lazy dog");
///
///     // Encode to hex
///     let mut hex_buffer = [0u8; 2usize * DEFAULT_DIGEST_SIZE];
///     encode_to_slice(&digest, &mut hex_buffer).unwrap();
///
///     // Print the digest (hex format)
///     println!("0x{}", str::from_utf8(&hex_buffer).unwrap());
/// }
/// ```
///
/// ### Context information
///
/// Optionally, additional “context” information may be provided via the `info` parameter:
///
/// ```rust
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute};
///
/// fn main() {
///     // Compute digest using the “one-shot” function with additional “info”
///     let digest: [u8; DEFAULT_DIGEST_SIZE] = compute(
///         Some("my_application"),
///         b"The quick brown fox jumps over the lazy dog");
///     /* ... */
/// }
/// ```
///
/// ### Important note
///
/// <div class="warning">
///
/// Applications that need to process *large* messages are recommended to use the [streaming API](SpongeHash256), which does **not** require *all* message data to be held in memory at once and which allows for an *incremental* hash computation.
///
/// </div>
pub fn compute<const N: usize, T: AsRef<[u8]>>(info: Option<&str>, message: T) -> [u8; N] {
    assert!(!info.is_some_and(str::is_empty), "Info must not be empty!");
    let mut state: SpongeHash256 = SpongeHash256::with_info(info.unwrap_or_default());
    state.update(message);
    state.digest()
}

/// Convenience function for “one-shot” SpongeHash-AES256 computation
///
/// The hash value (digest) of the given `message` is written into the slice `digest_out`.
///
/// A `message` can be of *any* type that implements the [`AsRef<[u8]>`](AsRef<T>) trait, e.g., `&[u8]`, `&str` or `String`.
///
/// Optionally, an additional `info` string may be specified.
///
/// The output slice is filled completely, generating a hash value (digest) of the appropriate size.
///
/// This function uses the default number of permutation rounds, as is given by [`DEFAULT_PERMUTE_ROUNDS`].
///
/// **Note:** The digest output size, i.e., `digest_out.len()`, in bytes, must be a *positive* value! &#x1F6A8;
///
/// ### Usage Example
///
/// The **`compute_to_slice()`** function can be used as follows:
///
/// ```rust
/// use hex::encode_to_slice;
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute_to_slice};
///
/// fn main() {
///     // Compute digest using the “one-shot” function
///     let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
///     compute_to_slice(&mut digest, None, b"The quick brown fox jumps over the lazy dog");
///
///     // Encode to hex
///     let mut hex_buffer = [0u8; 2usize * DEFAULT_DIGEST_SIZE];
///     encode_to_slice(&digest, &mut hex_buffer).unwrap();
///
///     // Print the digest (hex format)
///     println!("0x{}", str::from_utf8(&hex_buffer).unwrap());
/// }
///
/// ```
/// ### Context information
///
/// Optionally, additional “context” information may be provided via the `info` parameter:
///
/// ```rust
/// use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, compute_to_slice};
///
/// fn main() {
///     // Compute digest using the “one-shot” function with additional “info”
///     let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
///     compute_to_slice(
///             &mut digest,
///             Some("my_application"),
///             b"The quick brown fox jumps over the lazy dog");
///     /* ... */
/// }
/// ```
///
/// ### Important note
///
/// <div class="warning">
///
/// Applications that need to process *large* messages are recommended to use the [streaming API](SpongeHash256), which does **not** require *all* message data to be held in memory at once and which allows for an *incremental* hash computation.
///
/// </div>
pub fn compute_to_slice<T: AsRef<[u8]>>(digest_out: &mut [u8], info: Option<&str>, message: T) {
    assert!(!info.is_some_and(str::is_empty), "Info must not be empty!");
    let mut state: SpongeHash256 = SpongeHash256::with_info(info.unwrap_or_default());
    state.update(message);
    state.digest_to_slice(digest_out);
}
