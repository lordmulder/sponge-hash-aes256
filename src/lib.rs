// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#![no_std]

mod crypto;
mod sponge_hash;

pub use sponge_hash::{DEFAULT_DIGEST_SIZE, SpongeHash256, compute, compute_to_slice};
