// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use hex_literal::hex;
use rand_pcg::{
    rand_core::{Rng, SeedableRng},
    Pcg64Dxsm,
};
use regex::Regex;
use rolling_median::Median;
use std::{
    collections::BTreeSet,
    env::temp_dir,
    ffi::OsStr,
    fs::{create_dir, remove_dir_all, OpenOptions},
    io::{Error, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

fn create_random() -> Pcg64Dxsm {
    const SEED: [u8; 32usize] = hex!("2ca33785d2ae0c7fc0cf4c5267bf10f0854053c52428b24d3903a62c145a7f8b");
    Pcg64Dxsm::from_seed(SEED)
}

fn create_temp_folder(random: &mut Pcg64Dxsm) -> Result<PathBuf, Error> {
    let temp_dir = temp_dir();
    let mut error_counter = 0u16;
    loop {
        let temp_folder = temp_dir.join(format!("{:08X}.tmp", random.next_u32()));
        match create_dir(&temp_folder) {
            Ok(_) => return Ok(temp_folder),
            Err(error) => {
                error_counter += 1u16;
                if error_counter == u16::MAX {
                    return Err(error);
                }
            }
        }
    }
}

fn create_input_files(temp_folder: &Path, random: &mut Pcg64Dxsm) {
    // Create buffer
    let mut buffer = unsafe { Box::<[u8]>::new_uninit_slice(1048576usize).assume_init() };

    // Create data directories and files
    for _ in 0usize..SUBDIR {
        let subfolder = create_subfolder(temp_folder, random).expect("Failed to create subfolder!");
        for _ in 0usize..NFILES {
            create_data_file(&subfolder, &mut buffer, random).expect("Failed to create data file!");
        }
    }
}

fn create_subfolder(temp_folder: &Path, random: &mut Pcg64Dxsm) -> Result<PathBuf, Error> {
    let subfolder = temp_folder.join(format!("{:016X}", random.next_u64()));
    match create_dir(&subfolder) {
        Ok(_) => Ok(subfolder),
        Err(error) => Err(error),
    }
}

fn create_data_file(folder: &Path, buffer: &mut [u8], random: &mut Pcg64Dxsm) -> Result<(), Error> {
    let file_name = folder.join(format!("{:016X}{:016X}.dat", random.next_u64(), random.next_u64()));
    random.fill_bytes(buffer);
    let mut file = OpenOptions::new().write(true).create_new(true).open(file_name)?;
    file.write_all(buffer)
}

fn run_child_process(temp_folder: &Path, multi_thread: bool) -> Result<Option<String>, Error> {
    let mut args: Vec<&OsStr> = Vec::with_capacity(3usize);
    args.push(OsStr::new("--recursive"));
    if multi_thread {
        args.push(OsStr::new("--multi-threading"));
    }
    args.push(temp_folder.as_os_str());

    let command = Command::new(env!("CARGO_BIN_EXE_sponge256sum")).args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;

    let output = command.wait_with_output()?;
    Ok(if output.status.success() { Some(String::from_utf8_lossy(&output.stdout).into_owned()) } else { None })
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const PASSES_ST: usize = 11usize;
const PASSES_MT: usize = 31usize;

const SUBDIR: usize = 24usize;
const NFILES: usize = 256usize;

fn main() {
    // Print status
    println!("Generating test data, please wait...");

    // Initialize random
    let mut random = create_random();

    // Create temp directory
    let temp_folder = create_temp_folder(&mut random).expect("Failed to create temp folder!");

    // Create the input files
    create_input_files(&temp_folder, &mut random);

    // Print status
    println!("Completed.\n");

    // Prepare regular expression
    let regex_digest = Regex::new(r"^([0-9a-fA-F]+)\s([\x20-\x7E]+)$").expect("Failed to create regular expression!");

    for multi_thread in [false, true] {
        println!("[{}-benchmark]", if multi_thread { "MT" } else { "ST" });

        // Initialize median computation
        let mut median: Median<f64> = Median::new();

        // Number of passes
        let passes = if multi_thread { PASSES_MT } else { PASSES_ST };

        // Run the specified number of measuring passes
        for i in 0usize..=passes {
            if i != 0usize {
                println!("Measuring pass {:2} of {:2} is running, please wait...", i, passes);
            } else {
                println!("Warm-up pass is running...");
            }

            // Remember the start time
            let start_time = Instant::now();

            // Start the child process
            let output = run_child_process(&temp_folder, multi_thread).expect("Failed to start sub-process!");
            if output.is_none() {
                panic!("Process terminated with a non-zero exit code!");
            }

            // Compute elapsed time
            let elapsed = start_time.elapsed();

            // Parse the output
            let mut unique: BTreeSet<String> = BTreeSet::new();
            for captures in output.unwrap().lines().filter_map(|line| regex_digest.captures(line)) {
                unique.insert(captures.get(1usize).unwrap().as_str().to_ascii_lowercase());
            }

            // Assert number of unique hashes digest values
            if unique.len() != SUBDIR * NFILES {
                panic!("Number of unique hash values does not match!");
            }

            // Update median, unless this was the warm-up pass
            if i != 0usize {
                median.push(elapsed.as_secs_f64()).expect("Invalid elapsed time!");
            }
        }

        // Final output
        println!("--------\nMedian execution time: {:.2} seconds.\n", median.get().unwrap());
    }

    // Update status
    println!("Cleaning up test data, please wait...");

    // Remove data files
    remove_dir_all(temp_folder).expect("Failed to remove temporary files!");

    // Print status
    println!("Completed.");
}
