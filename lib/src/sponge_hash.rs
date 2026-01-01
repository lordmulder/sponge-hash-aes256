// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use crate::utilities::{length, Aes256Crypto, BlockType, BLOCK_SIZE};
use core::ops::Range;

/// Default digest size, in bytes
///
/// The default digest size is currently defined as **32** bytes, i.e., **256** bits.
pub const DEFAULT_DIGEST_SIZE: usize = 2usize * BLOCK_SIZE;

/// Default number of permutation rounds to be performed
///
/// The default number of permutation rounds is currently defined as **1**.
pub const DEFAULT_PERMUTE_ROUNDS: usize = 1usize;

/// Pre-define round keys
static ROUND_KEY_X: BlockType = BlockType::new::<0x5Cu8>();
static ROUND_KEY_Y: BlockType = BlockType::new::<0x36u8>();
static ROUND_KEY_Z: BlockType = BlockType::new::<0x6Au8>();

// ---------------------------------------------------------------------------
// Tracing
// ---------------------------------------------------------------------------

#[cfg(feature = "tracing")]
macro_rules! trace {
    ($self:tt, $arg:tt) => {
        log::trace!("SpongeHash256@{:p}: {} --> {:02X?} {:02X?} {:02X?}", &$self, $arg, &$self.state.0, &$self.state.1, &$self.state.2);
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
// Scratch buffer
// ---------------------------------------------------------------------------

/// Encapsulates the temporary computation state.
#[repr(align(32))]
struct Scratch {
    aes256: Aes256Crypto,
    temp: (BlockType, BlockType, BlockType),
}

impl Default for Scratch {
    fn default() -> Self {
        Self { aes256: Aes256Crypto::default(), temp: (BlockType::uninit(), BlockType::uninit(), BlockType::uninit()) }
    }
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
///     println!("0x{}", core::str::from_utf8(&hex_buffer).unwrap());
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
///
/// ### Algorithm
///
/// This section provides additional details about the SpongeHash-AES256 algorithm.
///
/// #### Internal state
///
/// The state has a total size of 384 bits, consisting of three 128-bit blocks, and is initialized to all zeros at the start of the computation. Only the upper 128 bits are directly used for input and output operations, as described below.
///
/// #### Update function
///
/// The “update” function, which *absorbs* input blocks into the state and *squeezes* the corresponding output from it, is defined as follows, where `input[i]` denotes the *i*-th input block and `output[k]` the *k*-th output block:
///
/// ![Update](https://github.com/lordmulder/sponge-hash-aes256/raw/master/.assets/images/function-update.png)
///
/// #### Permutation function
///
/// The “permutation” function, applied to scramble the state after each absorbing or squeezing step, is defined as follows, where `AES-256` denotes the ordinary [AES](https://en.wikipedia.org/wiki/Advanced_Encryption_Standard) block cipher with a key size of 256 bits and a block size of 128 bits.
///
/// ![Permutation](https://github.com/lordmulder/sponge-hash-aes256/raw/master/.assets/images/function-permutation.png)
///
/// The constants `const_0` and `const_1` are defined as full blocks filled with `0x5C` and `0x36`, respectively.
///
/// ### Finalization
///
/// The padding of the final input block is performed by first appending a single `1` bit, followed by the minimal number of `0` bits needed to make the total message length a multiple of the block size.
///
/// Following the final input block, a 128-bit block filled entirely with `0x6A` bytes is absorbed into the state.
#[repr(align(32))]
pub struct SpongeHash256<const R: usize = DEFAULT_PERMUTE_ROUNDS> {
    state: (BlockType, BlockType, BlockType),
    offset: usize,
}

impl<const R: usize> SpongeHash256<R> {
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
        let mut hash = Self { state: (BlockType::zero(), BlockType::zero(), BlockType::zero()), offset: 0usize };
        hash.initialize(info.as_bytes());
        hash
    }

    /// Initializes the internal state with the given `info` string
    #[inline]
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
    #[inline]
    pub fn update<T: AsRef<[u8]>>(&mut self, chunk: T) {
        trace!(self, "update::enter");

        let source = chunk.as_ref().as_ptr_range();
        if !source.is_empty() {
            unsafe {
                self.update_range(source);
            }
        }

        trace!(self, "update::leave");
    }

    /// Processes the next chunk of "raw" bytes, as specified by the [`Range<*const u8>`](slice::as_ptr_range) in the `source` parameter.
    ///
    /// The internal state of the hash computation is updated by this function.
    ///
    /// # Safety
    ///
    /// The caller **must** ensure that *all* byte addresses in the range from `source.start` up to but excluding `source.end` are valid!
    #[inline]
    pub unsafe fn update_range(&mut self, source: Range<*const u8>) {
        let mut source_next = source.start;
        let mut scratch_buffer = Scratch::default();

        while (self.offset != 0usize) && (source_next < source.end) {
            self.state.0[self.offset] ^= *source_next;
            self.offset += 1usize;
            source_next = source_next.add(1usize);

            if self.offset >= BLOCK_SIZE {
                self.permute(&mut scratch_buffer);
                self.offset = 0usize;
            }
        }

        if source_next < source.end {
            debug_assert_eq!(self.offset, 0usize);

            while length(source_next, source.end) >= BLOCK_SIZE {
                self.state.0.xor_with_u8_ptr(source_next);
                self.permute(&mut scratch_buffer);
                source_next = source_next.add(BLOCK_SIZE);
            }

            while source_next < source.end {
                self.state.0[self.offset] ^= *source_next;
                self.offset += 1usize;
                source_next = source_next.add(1usize);
            }
        }

        debug_assert!(self.offset < BLOCK_SIZE);
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

        let mut scratch_buffer = Scratch::default();

        self.state.0[self.offset] ^= 0x80u8;
        self.permute(&mut scratch_buffer);
        self.state.0.xor_with(&ROUND_KEY_Z);

        let mut pos = 0usize;

        while pos < digest_out.len() {
            self.permute(&mut scratch_buffer);
            let copy_len = BLOCK_SIZE.min(digest_out.len() - pos);
            digest_out[pos..(pos + copy_len)].copy_from_slice(&self.state.0[..copy_len]);
            pos += copy_len;
        }

        trace!(self, "digest::leave");
    }

    /// Pseudorandom permutation, based on the AES-256 block cipher
    #[inline]
    fn permute(&mut self, work: &mut Scratch) {
        trace!(self, "permfn::enter");

        for _ in 0..R {
            work.aes256.encrypt(&mut work.temp.0, &self.state.0, &self.state.1, &self.state.2);
            work.aes256.encrypt(&mut work.temp.1, &self.state.1, &self.state.2, &self.state.0);
            work.aes256.encrypt(&mut work.temp.2, &self.state.2, &self.state.0, &self.state.1);

            self.state.0.xor_with(&work.temp.0);
            self.state.1.xor_with(&work.temp.1);
            self.state.2.xor_with(&work.temp.2);

            self.state.1.xor_with(&ROUND_KEY_X);
            self.state.2.xor_with(&ROUND_KEY_Y);
        }

        trace!(self, "permfn::leave");
    }
}

impl Default for SpongeHash256 {
    fn default() -> Self {
        Self::new()
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
///     // Compute the digest using the “one-shot” function
///     let digest: [u8; DEFAULT_DIGEST_SIZE] = compute(
///         None,
///         b"The quick brown fox jumps over the lazy dog");
///
///     // Encode to hex
///     let mut hex_buffer = [0u8; 2usize * DEFAULT_DIGEST_SIZE];
///     encode_to_slice(&digest, &mut hex_buffer).unwrap();
///
///     // Print the digest (hex format)
///     println!("0x{}", core::str::from_utf8(&hex_buffer).unwrap());
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
///     // Compute the digest using the “one-shot” function with additional “info”
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
///     println!("0x{}", core::str::from_utf8(&hex_buffer).unwrap());
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
