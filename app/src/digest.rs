// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use sponge_hash_aes256::SpongeHash256;
use std::io::{BufRead, BufReader, Error as IoError, Read};

use crate::{
    arguments::Args,
    common::{Flag, MAX_SNAIL_LEVEL},
};

// ---------------------------------------------------------------------------
// Platform support
// ---------------------------------------------------------------------------

#[cfg(target_pointer_width = "64")]
const IO_BUFFER_SIZE: usize = 8192usize;
#[cfg(target_pointer_width = "32")]
const IO_BUFFER_SIZE: usize = 4096usize;
#[cfg(target_pointer_width = "16")]
const IO_BUFFER_SIZE: usize = 2048usize;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

pub enum Error {
    IoError,
    Cancelled,
}

impl From<IoError> for Error {
    fn from(_io_error: IoError) -> Self {
        Self::IoError
    }
}

// ---------------------------------------------------------------------------
// SpongeHash256 wrapper
// ---------------------------------------------------------------------------

const SNAIL_ITERATIONS_1: usize = 13usize;
const SNAIL_ITERATIONS_2: usize = 251usize;
const SNAIL_ITERATIONS_3: usize = 4093usize;
const SNAIL_ITERATIONS_4: usize = 65521usize;

enum Hasher {
    Default(SpongeHash256),
    SnailV1(SpongeHash256<SNAIL_ITERATIONS_1>),
    SnailV2(SpongeHash256<SNAIL_ITERATIONS_2>),
    SnailV3(SpongeHash256<SNAIL_ITERATIONS_3>),
    SnailV4(SpongeHash256<SNAIL_ITERATIONS_4>),
}

impl Hasher {
    #[inline(always)]
    pub fn new(info: &Option<String>, snail_level: u8) -> Self {
        assert!(snail_level <= MAX_SNAIL_LEVEL);
        match info {
            Some(info) => match snail_level {
                0u8 => Self::Default(SpongeHash256::with_info(info)),
                1u8 => Self::SnailV1(SpongeHash256::with_info(info)),
                2u8 => Self::SnailV2(SpongeHash256::with_info(info)),
                3u8 => Self::SnailV3(SpongeHash256::with_info(info)),
                4u8 => Self::SnailV4(SpongeHash256::with_info(info)),
                _ => unreachable!(),
            },
            None => match snail_level {
                0u8 => Self::Default(SpongeHash256::new()),
                1u8 => Self::SnailV1(SpongeHash256::new()),
                2u8 => Self::SnailV2(SpongeHash256::new()),
                3u8 => Self::SnailV3(SpongeHash256::new()),
                4u8 => Self::SnailV4(SpongeHash256::new()),
                _ => unreachable!(),
            },
        }
    }

    #[inline(always)]
    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        match self {
            Hasher::Default(hasher) => hasher.update(input),
            Hasher::SnailV1(hasher) => hasher.update(input),
            Hasher::SnailV2(hasher) => hasher.update(input),
            Hasher::SnailV3(hasher) => hasher.update(input),
            Hasher::SnailV4(hasher) => hasher.update(input),
        }
    }

    #[inline(always)]
    pub fn digest_to_slice(self, output: &mut [u8]) {
        match self {
            Hasher::Default(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV1(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV2(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV3(hasher) => hasher.digest_to_slice(output),
            Hasher::SnailV4(hasher) => hasher.digest_to_slice(output),
        }
    }
}

// ---------------------------------------------------------------------------
// Compute digest
// ---------------------------------------------------------------------------

/// Check if the computation has been aborted
macro_rules! check_cancelled {
    ($halt:ident) => {
        if !$halt.running() {
            return Err(Error::Cancelled);
        }
    };
}

/// Process a single input file
pub fn compute_digest(input: &mut dyn Read, digest_out: &mut [u8], args: &Args, halt: &Flag) -> Result<(), Error> {
    static LINE_BREAK: &str = "\n";
    let mut hasher = Hasher::new(&args.info, args.snail);

    if !args.text {
        let mut buffer = [0u8; IO_BUFFER_SIZE];
        loop {
            check_cancelled!(halt);
            match input.read(&mut buffer)? {
                0 => break,
                length => hasher.update(&buffer[..length]),
            }
        }
    } else {
        let mut lines = BufReader::new(input).lines();
        if let Some(line) = lines.next() {
            hasher.update(&(line?));
            for line in lines {
                check_cancelled!(halt);
                hasher.update(LINE_BREAK);
                hasher.update(&(line?));
            }
        }
    }

    hasher.digest_to_slice(digest_out);
    Ok(())
}

// ---------------------------------------------------------------------------
// Verify digest
// ---------------------------------------------------------------------------

#[inline]
pub fn digest_equal(digest0: &[u8], digest1: &[u8]) -> bool {
    assert_eq!(digest0.len(), digest1.len(), "Digest size mismatch!");
    let mut mask = 0u8;
    for (value0, value1) in digest0.iter().zip(digest1.iter()) {
        mask |= value0 ^ value1;
    }
    mask == 0u8
}
