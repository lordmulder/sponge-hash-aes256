// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use rolling_median::Median;
use sponge_hash_aes256::{SpongeHash256, DEFAULT_DIGEST_SIZE};
use std::{hint::black_box, time::Instant};

// ---------------------------------------------------------------------------
// Macros
// ---------------------------------------------------------------------------

const ITERATIONS: usize = 100usize;
const SAMPLES: usize = 0xFFFFusize;

macro_rules! measure {
    ($function:expr) => {
        let mut measurement = Measurement::new();
        for _i in 0usize..SAMPLES {
            black_box($function(black_box(&mut measurement)));
        }
        println!("{:.10} -- {}", measurement.result(), stringify!($function).strip_prefix("perf_").unwrap());
    };
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

struct Measurement {
    rolling_median: Median<f64>,
}

impl Measurement {
    pub fn new() -> Self {
        Self { rolling_median: Median::new() }
    }

    #[inline]
    pub fn run<R, F: Fn() -> R>(&mut self, function: F) {
        let timestamp_start = Instant::now();
        for _i in 0usize..ITERATIONS {
            black_box(function());
        }
        let duration = timestamp_start.elapsed().as_secs_f64() / (ITERATIONS as f64);
        self.rolling_median.push(duration).unwrap();
    }

    #[inline]
    pub fn run_mut<T, R, F: Fn(&mut T) -> R>(&mut self, param: &mut T, function: F) {
        let timestamp_start = Instant::now();
        for _i in 0usize..ITERATIONS {
            black_box(function(black_box(param)));
        }
        let duration = timestamp_start.elapsed().as_secs_f64() / (ITERATIONS as f64);
        self.rolling_median.push(duration).unwrap();
    }

    #[inline]
    pub fn run_cloned<T: Clone, R, F: Fn(T) -> R>(&mut self, param: T, function: F) {
        let timestamp_start = Instant::now();
        for _i in 0usize..ITERATIONS {
            black_box(function(black_box(param.clone())));
        }
        let duration = timestamp_start.elapsed().as_secs_f64() / (ITERATIONS as f64);
        self.rolling_median.push(duration).unwrap();
    }

    pub fn result(self) -> f64 {
        self.rolling_median.get().unwrap_or(f64::MAX)
    }
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

fn perf_spongehash256_new(measurement: &mut Measurement) {
    measurement.run(|| {
        let instance = SpongeHash256::default();
        black_box(instance)
    });
}

fn perf_spongehash256_with_info(measurement: &mut Measurement) {
    measurement.run(|| {
        let instance = SpongeHash256::<1usize>::with_info(black_box("Hellorld!"));
        black_box(instance)
    });
}

fn perf_spongehash256_update_empty(measurement: &mut Measurement) {
    let mut instance = SpongeHash256::default();
    measurement.run_mut(&mut instance, |hash| {
        hash.update(black_box(b""));
    });
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    black_box(digest);
}

fn perf_spongehash256_update_tiny(measurement: &mut Measurement) {
    let mut instance = SpongeHash256::default();
    measurement.run_mut(&mut instance, |hash| {
        hash.update(black_box(b"a"));
    });
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    black_box(digest);
}

fn perf_spongehash256_update_small(measurement: &mut Measurement) {
    let mut instance = SpongeHash256::default();
    measurement.run_mut(&mut instance, |hash| {
        hash.update(black_box(b"abcdefghijklmn"));
    });
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    black_box(digest);
}

fn perf_spongehash256_update_big(measurement: &mut Measurement) {
    let mut instance = SpongeHash256::default();
    measurement.run_mut(&mut instance, |hash| {
        hash.update(black_box(b"P9duhSwFiQFTSUMdBks0xc01Vjwxzu4TCnrhjt4i5XwiZSlIgSklnwxVnYNj2ruK"));
    });
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    black_box(digest);
}

fn perf_spongehash256_update_huge(measurement: &mut Measurement) {
    let mut instance = SpongeHash256::default();
    measurement.run_mut(&mut instance, |hash| {
        hash.update(black_box(b"11VALp5IyqDmZOQmW6FiRtyINoCjIfI5CfcFPqyyiC1IN4AHyYvi9JTNqasKYQMNKftbFenmWWJaN877bbbX4pleqmWdd9lZFx0vbLOOjuSJ7RQLztVfeL9ytx6N5Bkswy6YW5f2DczeU6L6xAzNWtIQDOGv7lfZuCJ6xqlju1cEj7dKwG9GHoTQkPyMQJrnG1njGFB9Gsdg2C3vqzEBPbEMjmCj7PhQLNkx2qbCWFc8oRhI9ULYG6F2Lv9F08IzOtOCDJZ4SD3D8C21Jr0qSBSKs4hVWRejdAxVjySSS8WoS90ZLFvliofbDkQFiE4u01aaEYu7Gxj251G8jAD7e4hTzhB5sFeInlYQEg0Gj8h1pQfbFLL4QsXgr7g5SNtceJLdkd0YxyLTrSKyCTXFY5YGxaY3dEaT0ybBZjn78PDFnEONjMvjOQb0nu8TH9K4NSDz4XFeQbge041qKsFugFrLHxziilPLizGwmcfU8Z67AkaHSph1VICavPGLkCLhdtLlSJhO9U6a8dD1YCNQF6l36AVMyr10XfamEz40Wq7XRGIsRto5PgOOL525WQ9NKAXwxhddGyTkAj6N0TwRFizeIBIk7ch1L45nNYQZGMyeaQMCfKENeYD1qsSFDpSb"));
    });
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    black_box(digest);
}

fn perf_spongehash256_digest(measurement: &mut Measurement) {
    let mut instance = SpongeHash256::default();
    instance.update(black_box(b"P9duhSwFiQFTSUMdBks0xc01Vjwxzu4TCnrhjt4i5XwiZSlIgSklnwxVnYNj2ruK"));
    measurement.run_cloned(instance, |hash| {
        let digest: [u8; DEFAULT_DIGEST_SIZE] = hash.digest();
        black_box(digest)
    });
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    measure!(perf_spongehash256_new);
    measure!(perf_spongehash256_with_info);
    measure!(perf_spongehash256_update_empty);
    measure!(perf_spongehash256_update_tiny);
    measure!(perf_spongehash256_update_small);
    measure!(perf_spongehash256_update_big);
    measure!(perf_spongehash256_update_huge);
    measure!(perf_spongehash256_digest);
}
