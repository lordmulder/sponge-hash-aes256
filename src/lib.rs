// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#![no_std]

//! # SpongeHash-AES256
//!
//! A [**sponge**](https://en.wikipedia.org/wiki/Sponge_function)-based secure hash function that uses [AES-256](https://docs.rs/aes/latest/aes/index.html) as its internal [PRF](https://en.wikipedia.org/wiki/Pseudorandom_permutation).
//!
//! This hash function has a *variable* output size and can produce outputs of *any* size in the range from 8 to 16,384 (inclusive) bits.
//!
//! Please see the [`SpongeHash256`] struct for details!
//!
//! # License
//!
//! Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;
//!
//! Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
mod sponge_hash;
mod utilities;

pub use sponge_hash::{DEFAULT_DIGEST_SIZE, SpongeHash256, compute, compute_to_slice};
