// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#![no_std]
#![allow(clippy::needless_doctest_main)]

//! # SpongeHash-AES256
//!
//! A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).
//!
//! This hash function has a *variable* output size and can produce outputs of *any* non-zero size (up to [`usize::MAX`]).
//!
//! Please see the [`SpongeHash256`] struct for details! &#128161;
//!
//! ## Dependencies
//!
//! This crate is **`#![no_std]`** compatible and does not link the Rust standard library.
//!
//! Required dependencies: [`aes`](https://crates.io/crates/aes), [`cipher`](https://crates.io/crates/cipher), [`zeroize`](https://crates.io/crates/zeroize)
//!
//! ## Optional features
//!
//! Feature   | Meaning
//! --------- | ------------------------------------------------------------------------------------------
//! `tracing` | Dump the internal state to the loggging sub-system (via `log::trace()`) after each step.
//!
//! ## Rust support
//!
//! This crate uses Rust edition 2021, and requires `rustc` version 1.78.0 or newer.
//!
//! ## License
//!
//! Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;
//!
//! Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

mod sponge_hash;
mod utilities;

pub use sponge_hash::{compute, compute_to_slice, SpongeHash256, DEFAULT_DIGEST_SIZE, DEFAULT_PERMUTE_ROUNDS};
pub use utilities::version;
