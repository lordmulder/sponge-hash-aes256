// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use hex_literal::hex;
use rand_pcg::{
    rand_core::{Rng, SeedableRng},
    Pcg64,
};
use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

// ---------------------------------------------------------------------------
// Randomness
// ---------------------------------------------------------------------------

struct RandContext {
    burned: HashSet<u64>,
    random: Pcg64,
}

impl RandContext {
    pub fn new() -> Self {
        let mut seed = hex!("2ca33785d2ae0c7fc0cf4c5267bf10f0854053c52428b24d3903a62c145a7f8b");
        for (index, value) in SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_be_bytes().iter().enumerate() {
            seed[16usize + index] ^= value;
        }
        Self { burned: HashSet::new(), random: Pcg64::from_seed(seed) }
    }
}

static RANDOM: LazyLock<Mutex<RandContext>> = LazyLock::new(|| Mutex::new(RandContext::new()));

pub fn random_u64() -> u64 {
    let mut context = RANDOM.lock().unwrap();

    loop {
        let value = context.random.next_u64();
        if context.burned.insert(value) {
            return value;
        }
    }
}
