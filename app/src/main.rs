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
//!   -m, --multi-threading  Enable multi-threaded processing of input files
//!   -f, --flush            Explicitly flush 'stdout' stream after printing a digest
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
//!   –      | 1 (default)                  |            249,245.54
//!   **×1** | 13                           |              8,595.85
//!   **×2** | 251                          |                441.40
//!   **×3** | 4093                         |                 25.82
//!   **×4** | 65521                        |                  1.61
//!
//! - **Text mode**
//!
//!   The `--text` option enables “text” mode. In this mode, the input file is read as a *text* file, line by line.
//!
//!   Unlike in “binary” mode (the default), platform-specific line endings will be normalized to a single `\n` character.
//!
//! - **Multi-threading**
//!
//!   The `--multi-threading` option enables [multithreading](https://en.wikipedia.org/wiki/Thread_(computing)) mode, in which multiple files can be processed concurrently.
//!
//!   Note that, in this mode, the order in which the files will be processed is ***undefined***.
//!
//!   Also note that each file still is processed by a single thread, so this mode is only useful when processing *many* files.
//!
//! ## Environment
//!
//! The following environment variables are recognized:
//!
//! - **`SPONGE256SUM_THREAD_COUNT`**:  
//!   Specifies the number of threads to be used in `--multi-threading` mode.  
//!   If set to **0**, which is the default, the number of CPU cores is detected automatically at runtime.  
//!   Please note that the number of threads is currently limited to the range from 1 to 32.
//!
//! - **`SPONGE256SUM_DIRWALK_STRATEGY`**:  
//!   Selects the search strategy to be used for walking the directory tree in `--recursive` mode.  
//!   This can be `BFS` (breadth-first search) or `DFS` (depath-first search). Default is `BFS`.
//!
//! - **`SPONGE256SUM_SELFTEST_PASSES`**:  
//!   Specifies the number of passes to be executed in `--self-test` mode. Default is **3**.
//!
//! ## Platform support
//!
//! This crate uses Rust edition 2021, and requires `rustc` version 1.89.0 or newer.
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
//!
//! ## See also
//!
//! &#x1F517; <https://crates.io/crates/sponge-hash-aes256>  
//! &#x1F517; <https://github.com/lordmulder/sponge-hash-aes256>

mod arguments;
mod common;
mod digest;
mod environment;
mod io;
mod process;
mod self_test;
mod thread_pool;
mod verify;

use num::Integer;
use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::process::abort;
use std::thread;
use std::time::Duration;
use std::{io::stdout, process::ExitCode, sync::Arc};

use crate::common::{Aborted, Flag};
use crate::environment::Env;
use crate::verify::verify_files;
use crate::{
    arguments::Args,
    common::{MAX_DIGEST_SIZE, MAX_SNAIL_LEVEL},
    process::process_files,
    self_test::self_test,
};

// Enable MiMalloc, if the "with-mimalloc" feature is enabled
#[cfg(feature = "with-mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// The actual "main" function
fn sponge256sum_main(args: Arc<Args>) -> Result<bool, Aborted> {
    // Initialize the SimpleLogger, if the "with-logging" feature is enabled
    #[cfg(feature = "with-logging")]
    simple_logger::SimpleLogger::new().init().unwrap();

    // Compute the digest size, in bytes (falling back to the default, it unspecified)
    let (digest_size, digest_rem) = match args.length {
        Some(digest_bits) => digest_bits.get().div_rem(&(u8::BITS as usize)),
        None => (DEFAULT_DIGEST_SIZE, 0usize),
    };

    // Make sure that the digest size is divisble by eight
    if digest_rem != 0usize {
        print_error!(args, "Error: Digest output size must be divisible by eight! (given value: {}, remainder: {})", args.length.unwrap().get(), digest_rem);
        return Ok(false);
    }

    // Make sure that the digest size doesn't exceed the allowable maximum
    if digest_size > MAX_DIGEST_SIZE {
        print_error!(args, "Error: Digest output size exceeds the allowable maximum! (given value: {})", digest_size * 8usize);
        return Ok(false);
    }

    // Check for snail level being out of bounds
    if args.snail > MAX_SNAIL_LEVEL {
        print_error!(args, "\n{}", include_str!("../../.assets/text/goat.txt"));
        return Ok(false);
    }

    // Check the maximum allowable info length
    if args.info.as_ref().is_some_and(|str| str.len() > u8::MAX as usize) {
        print_error!(args, "Error: Length of context info must not exceed 255 characters! (given length: {})", args.info.as_ref().unwrap().len());
        return Ok(false);
    }

    // Parse additional options from environment variables
    let env = match Env::from_env() {
        Ok(options) => options,
        Err(error) => {
            print_error!(args, "Error: Environment variable {}={:?} is invalid!", error.name, error.value);
            return Ok(false);
        }
    };

    // Install the interrupt (CTRL+C) handling routine
    let halt = Arc::new(Default::default());
    let halt_cloned = Arc::clone(&halt);
    let _ctrlc = ctrlc::set_handler(move || ctrlc_handler(&halt_cloned));

    // Acquire stdout handle
    let mut output = stdout().lock();

    // Run built-in self-test, if it was requested by the user
    if args.self_test {
        self_test(&mut output, &args, &env, &halt)
    } else if !args.check {
        // Process all input files/directories that were given on the command-line
        process_files(&mut output, digest_size, args, &env, halt)
    } else {
        // Verify all checksum files that were given on the command-line
        verify_files(&mut output, args, &env, halt)
    }
}

// ---------------------------------------------------------------------------
// Interrupt handler
// ---------------------------------------------------------------------------

/// The SIGINT (CTRL+C) interrupt handler routine
///
/// If the process does not exit cleanly after 10 seconds, we just proceed with the abort!
fn ctrlc_handler(halt: &Arc<Flag>) -> ! {
    let _ = halt.abort_process();
    thread::sleep(Duration::from_secs(10u64));
    abort();
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Applicationm entry point (“main” function)
fn main() -> ExitCode {
    // Initialize the Args from the given command-line arguments
    let args = match Args::try_parse_command_line() {
        Ok(args) => Arc::new(args),
        Err(exit_code) => return exit_code,
    };

    // Call the actual "main" function
    match sponge256sum_main(Arc::clone(&args)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(Aborted) => {
            print_error!(args, "Aborted: The process has been interrupted by the user!");
            ExitCode::from(130u8)
        }
    }
}
