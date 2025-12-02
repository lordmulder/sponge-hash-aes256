// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#![doc(hidden)]

//! # sponge256sum
//!
//! A command-line tool for computing [**SpongeHash-AES256**](https://github.com/lordmulder/sponge-hash-aes256/) message digest.
//!
//! This program is designed as a drop-in replacement for [`sha1sum`](https://manpages.debian.org/trixie/coreutils/sha1sum.1.en.html), [`sha256sum`](https://manpages.debian.org/trixie/coreutils/sha256sum.1.en.html) and related utilities.
//!
//! Please see the [library documentation](sponge_hash_aes256) for details! &#128161;
//!
//! ## Usage
//!
//! This command-line application can be used as follows:
//!
//! ```plaintext
//! Usage: sponge256sum [OPTIONS] [FILES]...
//!
//! Arguments:
//!   [FILES]...  Files to be processed
//!
//! Options:
//!   -b, --binary           Read the input file(s) in binary mode, i.e., default mode
//!   -t, --text             Read the input file(s) in text mode
//!   -c, --check            Read and verify checksums from the provided input file(s)
//!   -d, --dirs             Enable processing of directories as arguments
//!   -r, --recursive        Recursively process the provided directories (implies -d)
//!   -k, --keep-going       Continue processing even if errors are encountered
//!   -l, --length <LENGTH>  Digest output size, in bits (default: 256, maximum: 2048)
//!   -i, --info <INFO>      Include additional context information
//!   -s, --snail...         Enable "snail" mode, i.e., slow down the hash computation
//!   -q, --quiet            Do not output any error messages or warnings
//!   -p, --plain            Print digest(s) in plain format, i.e., without file names
//!   -0, --null             Separate digest(s) by NULL characters instead of newlines
//!   -f, --flush            Explicitely flush 'stdout' stream after printing a digest
//!   -T, --self-test        Run the built-in self-test (BIST)
//!   -h, --help             Print help
//!   -V, --version          Print version
//!
//! If no input files are specified, reads input data from the 'stdin' stream.
//! Returns a non-zero exit code if any errors occurred; otherwise, zero.
//! ```
//!
//! ## Examples
//!
//! Here are some `sponge256sum` usage examples:
//!
//! * Compute the hash values (digests) of one or multiple input files:
//!   ```sh
//!   $ sponge256sum /path/to/first.dat /path/to/second.dat /path/to/third.dat
//!   ```
//!
//! * Selecting multiple input files can also be done with wildcards:
//!   ```sh
//!   $ sponge256sum /path/to/*.dat
//!   ```
//!
//! * Perform a recursive scan of an entire directory tree:
//!   ```sh
//!   $ sponge256sum --recursive /path/to/base-dir
//!   ```
//!
//! * Compute the hash value (digest) of the data from the `stdin` stream:
//!   ```sh
//!   $ printf "Lorem ipsum dolor sit amet consetetur sadipscing" | sponge256sum
//!   ```
//!
//! * Verify files (hashes) from an existing checksum file:
//!   ```sh
//!   $ sponge256sum --check /path/to/SPONGE256SUMS.txt
//!   ```
//!
//! ## Options
//!
//! The following options are available, among others:
//!
//! - **Output length**
//!
//!   The `--length <LENGTH>` option can be used to specify the digest output size, in bits. The default size is 256 bits.
//!
//!   Currently, the maximum output size is 1024 bits. Also, the output size, in bits, must be divisible by eight!
//!
//! - **Context information**
//!
//!   The `--info <INFO>` option can be used to include some additional context information in the hash computation.
//!
//!   For each unique “info” string, different digests (hash values) are generated from the same messages (inputs).
//!
//!   This enables proper *domain separation* for different uses, e.g., applications or protocols, of the same hash function.
//!
//! - **Snail mode**
//!
//!   The `--snail` option can be passed to the program, optionally more than once, to slow down the hash computation.
//!
//!   This improves the security of certain applications, e.g., password hashing, by making “brute force” attacks harder.
//!
//!   Count  | Number of permutation rounds | Throughput (in KiB/s)
//!   ------ | ---------------------------- | --------------------:
//!   –      | 1 (default)                  |             40,013.31
//!   **×1** | 13                           |              2,713.64
//!   **×2** | 251                          |                138.44
//!   **×3** | 4093                         |                  8.88
//!   **×4** | 65521                        |                  0.55
//!
//! - **Text mode**
//!
//!   The `--text` option enables “text” mode. In this mode, the input file is read as a *text* file, line by line.
//!
//!   Unlike in “binary” mode (the default), platform-specific line endings will be normalized to a single `\n` character.
//!
//! ## Environment
//!
//! The following environment variables are recognized:
//!
//! - **`SPONGE256SUM_SELFTEST_PASSES`**:  
//!   Specifies the number of passes to be executed in “self-test” mode. Default is **3**.
//!
//! - **`SPONGE256SUM_DIRWALK_STRATEGY`**:  
//!   Selects the search strategy to be used for walking the directory tree in “recursive” mode.  
//!   This can be `BFS` (breadth-first search) or `DFS` (depath-first search). Default is `BFS`.
//!
//! ## Platform support
//!
//! This crate uses Rust edition 2021, and requires `rustc` version 1.78.0 or newer.
//!
//! The following targets are officially supported, other platforms may function but are **not** guaranteed:
//!
//! - Linux
//! - Windows
//! - macOS
//! - *BSD (FreeBSD, OpenBSD, NetBSD, etc.)
//! - Haiku OS
//! - Solaris / Illumos
//!
//! ## License
//!
//! Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;
//!
//! Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

mod arguments;
mod common;
mod digest;
mod io;
mod process;
mod process_files;
mod self_test;
mod verify;

use num::Integer;
use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{io::stdout, process::ExitCode, sync::Arc};

use crate::process::process_from_stdin;
use crate::verify::{verify_files, verify_from_stdin};
use crate::{
    arguments::Args,
    common::{MAX_DIGEST_SIZE, MAX_SNAIL_LEVEL},
    process_files::process_files,
    self_test::self_test,
};

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// Applicationm entry point (“main” function)
fn main() -> ExitCode {
    // Initialize the Args from the given command-line arguments
    let args = Arc::new(Args::parse_command_line());

    // Check for incompatible arguments
    if args.text && args.binary {
        print_error!(args, "Error: Options '--binary' and '--text' are mutually exclusive!");
        return ExitCode::FAILURE;
    }

    // Check for options that cannot be used in `--check` mode
    if args.check && args.length.is_some() {
        print_error!(args, "Error: Option '--length' must not be used in '--check' mode!");
        return ExitCode::FAILURE;
    }

    // Compute the digest size, in bytes (falling back to the default, it unspecified)
    let (digest_size, digest_rem) = match args.length {
        Some(digest_bits) => digest_bits.get().div_rem(&(u8::BITS as usize)),
        None => (DEFAULT_DIGEST_SIZE, 0usize),
    };

    // Make sure that the digest size is divisble by eight
    if digest_rem != 0usize {
        print_error!(args, "Error: Digest output size must be divisble by eight! (given value: {})", args.length.unwrap().get());
        return ExitCode::FAILURE;
    }

    // Make sure that the digest size doesn't exceed the allowable maximum
    if digest_size > MAX_DIGEST_SIZE {
        print_error!(args, "Error: Digest output size exceeds the allowable maximum! (given value: {})", digest_size * 8usize);
        return ExitCode::FAILURE;
    }

    // Check for snail level being out of bounds
    if args.snail > MAX_SNAIL_LEVEL {
        print_error!(args, "\n{}", include_str!("../../.assets/text/goat.txt"));
        return ExitCode::FAILURE;
    }

    // Check the maximum allowable info length
    if args.info.as_ref().is_some_and(|str| str.len() > u8::MAX as usize) {
        print_error!(args, "Error: Length of \"info\" must not exceed 255 characters! (given length: {})", args.info.as_ref().unwrap().len());
        return ExitCode::FAILURE;
    }

    // Install the interrupt (CTRL+C) handling routine
    let halt = Arc::new(AtomicBool::new(false));
    let halt_cloned = halt.clone();
    let _ = ctrlc::set_handler(move || halt_cloned.store(true, Ordering::SeqCst));

    // Acquire stdout handle
    let mut output = stdout().lock();

    // Run built-in self-test, if it was requested by the user
    let success = if args.self_test {
        self_test(&mut output, &args, &halt)
    } else if args.check {
        if args.files.is_empty() {
            verify_from_stdin(&mut output, &args, &halt)
        } else {
            verify_files(args.files.iter(), &mut output, &args, &halt)
        }
    } else {
        // Process all files and directories that were given on the command-line
        if args.files.is_empty() {
            process_from_stdin(&mut output, digest_size, &args, &halt)
        } else {
            process_files(&mut output, digest_size, &args, &halt)
        }
    };

    if success {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
