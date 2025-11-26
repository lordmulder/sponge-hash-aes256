// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use rolling_median::Median;
use sponge_hash_aes256::{SpongeHash256, DEFAULT_DIGEST_SIZE};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Macros
// ---------------------------------------------------------------------------

macro_rules! measure {
    ($func:expr) => {
        let name = stringify!($func);
        println!("{:.9} -- {}", measure_function($func), name.strip_prefix("perf_").unwrap_or(name));
    };
}

// ---------------------------------------------------------------------------
// Utilities
// ---------------------------------------------------------------------------

const PASSES: usize = 0x1FFFFFusize;

fn measure_function<T: Fn()>(function: T) -> f64 {
    let mut rolling_median = Median::new();

    for _i in 0usize..=PASSES {
        let start_time = Instant::now();
        function();
        let duration = start_time.elapsed();
        rolling_median.push(duration.as_secs_f64());
    }

    rolling_median.get().unwrap_or(f64::MAX)
}

#[inline(always)]
fn read_volatile<T>(instance: &T) {
    unsafe {
        let _value = (instance as *const T).read_volatile();
    }
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

fn perf_spongehash256_new() {
    let instance = SpongeHash256::default();
    read_volatile(&instance);
}

fn perf_spongehash256_with_info() {
    let instance = SpongeHash256::<1usize>::with_info("Hellorld!");
    read_volatile(&instance);
}

fn perf_spongehash256_update_empty() {
    let mut instance = SpongeHash256::default();
    instance.update(b"");
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    read_volatile(&digest);
}

fn perf_spongehash256_update_tiny() {
    let mut instance = SpongeHash256::default();
    instance.update(b"a");
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    read_volatile(&digest);
}

fn perf_spongehash256_update_small() {
    let mut instance = SpongeHash256::default();
    instance.update(b"abcdefghijklmn");
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    read_volatile(&digest);
}

fn perf_spongehash256_update_big() {
    let mut instance = SpongeHash256::default();
    instance.update(b"P9duhSwFiQFTSUMdBks0xc01Vjwxzu4TCnrhjt4i5XwiZSlIgSklnwxVnYNj2ruK");
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    read_volatile(&digest);
}

fn perf_spongehash256_update_huge() {
    let mut instance = SpongeHash256::default();
    instance.update(b"11VALp5IyqDmZOQmW6FiRtyINoCjIfI5CfcFPqyyiC1IN4AHyYvi9JTNqasKYQMNKftbFenmWWJaN877bbbX4pleqmWdd9lZFx0vbLOOjuSJ7RQLztVfeL9ytx6N5Bkswy6YW5f2DczeU6L6xAzNWtIQDOGv7lfZuCJ6xqlju1cEj7dKwG9GHoTQkPyMQJrnG1njGFB9Gsdg2C3vqzEBPbEMjmCj7PhQLNkx2qbCWFc8oRhI9ULYG6F2Lv9F08IzOtOCDJZ4SD3D8C21Jr0qSBSKs4hVWRejdAxVjySSS8WoS90ZLFvliofbDkQFiE4u01aaEYu7Gxj251G8jAD7e4hTzhB5sFeInlYQEg0Gj8h1pQfbFLL4QsXgr7g5SNtceJLdkd0YxyLTrSKyCTXFY5YGxaY3dEaT0ybBZjn78PDFnEONjMvjOQb0nu8TH9K4NSDz4XFeQbge041qKsFugFrLHxziilPLizGwmcfU8Z67AkaHSph1VICavPGLkCLhdtLlSJhO9U6a8dD1YCNQF6l36AVMyr10XfamEz40Wq7XRGIsRto5PgOOL525WQ9NKAXwxhddGyTkAj6N0TwRFizeIBIk7ch1L45nNYQZGMyeaQMCfKENeYD1qsSFDpSb");
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    read_volatile(&digest);
}

fn perf_spongehash256_update_loop_small() {
    let mut instance = SpongeHash256::default();
    for _k in 0..97usize {
        instance.update(b"RupmEJv9taIb2s9vKTxgSfQU06o4Nfl");
    }
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    read_volatile(&digest);
}

fn perf_spongehash256_update_loop_huge() {
    let mut instance = SpongeHash256::default();
    for _k in 0..5usize {
        instance.update(b"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");
    }
    let digest: [u8; DEFAULT_DIGEST_SIZE] = instance.digest();
    read_volatile(&digest);
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
    measure!(perf_spongehash256_update_loop_small);
    measure!(perf_spongehash256_update_loop_huge);
}
