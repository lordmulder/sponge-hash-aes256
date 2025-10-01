// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

//! # sponge256sum
//!
//! A command-line tool for computing [**SpongeHash-AES256**](https://github.com/lordmulder/sponge-hash-aes256/) message digest.
//!
//! This program is designed as a drop-in replacement for [`sha1sum`](https://manpages.debian.org/trixie/coreutils/sha1sum.1.en.html), [`sha256sum`](https://manpages.debian.org/trixie/coreutils/sha256sum.1.en.html) and related utilities.
//!
//! Please see the [library documentation](https://docs.rs/sponge-hash-aes256/latest/) for details! &#128161;
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
//!   -k, --keep-going       Keep going, even if an input file can not be read
//!   -l, --length <LENGTH>  Digest output size, in bits (default: 256, maximum: 1024)
//!   -i, --info <INFO>      Include additional context information
//!   -s, --snail            Enable "snail" mode, i.e., slow down the hash computation
//!   -q, --quiet            Do not output any error messages or warnings
//!   -h, --help             Print help
//!   -V, --version          Print version
//!
//! If no input files are specified, reads input data from 'stdin' stream.
//! ```
//!
//! ## Examples
//!
//! Here are some `sponge256sum` usage examples:
//!
//! * Compute the hash values (digests) of one or multiple input files:
//!   ```sh
//!   $ sponge256sum /path/to/foo.dat /path/to/bar.dat /path/to/baz.dat
//!   ```
//!
//! * Selecting multiple input files can also be done with wildcards:
//!   ```sh
//!   $ sponge256sum /path/to/*.dat
//!   ```
//!
//! * Compute the hash value (digest) of the data from the `stdin` stream:
//!   ```sh
//!   $ printf "Lorem ipsum dolor sit amet consetetur sadipscing" | sponge256sum
//!   ```
//!
//! ## License
//!
//! Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;
//!
//! Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use clap::{Parser, command};
use const_format::formatcp;
use hex::encode_to_slice;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, PKG_VERSION as LIB_VERSION, SpongeHash256};
use std::{
    env::consts::{ARCH, OS},
    fs::File,
    io::{BufRead, BufReader, Error, Read, stdin},
    num::NonZeroUsize,
    process::ExitCode,
    slice::Iter,
};

/// Maximum allowable digest size, specified in bytes
const MAX_DIGEST_SIZE: usize = 128usize;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Build profile
#[cfg(debug_assertions)]
const BUILD_PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
const BUILD_PROFILE: &str = "release";

/// Header line
const HEADER_LINE: &str =
    formatcp!("{} v{} (with SpongeHash-AES256 v{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), LIB_VERSION);

/// Version string
const VERSION_STR: &str =
    formatcp!("v{} [SpongeHash-AES256 v{}] [{}] [{}] [{}]", env!("CARGO_PKG_VERSION"), LIB_VERSION, OS, ARCH, BUILD_PROFILE);

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// SpongeHash-AES256 command-line tool
#[derive(Parser, Debug)]
#[command(about = "A sponge-based secure hash function that uses AES-256 as its internal PRF.")]
#[command(after_help = "If no input files are specified, reads input data from 'stdin' stream.")]
#[command(version = VERSION_STR)]
#[command(before_help = HEADER_LINE)]
struct Args {
    /// Read the input file(s) in binary mode, i.e., default mode
    #[arg(short, long)]
    binary: bool,

    /// Read the input file(s) in text mode
    #[arg(short, long)]
    text: bool,

    /// Keep going, even if an input file can not be read
    #[arg(short, long)]
    keep_going: bool,

    /// Digest output size, in bits (default: 256, maximum: 1024)
    #[arg(short, long)]
    length: Option<NonZeroUsize>,

    /// Include additional context information
    #[arg(short, long)]
    info: Option<String>,

    /// Enable "snail" mode, i.e., slow down the hash computation
    #[arg(short, long)]
    snail: bool,

    /// Do not output any error messages or warnings
    #[arg(short, long)]
    quiet: bool,

    /// Print digest(s) in plain format, i.e., without file names
    #[arg(short, long)]
    plain: bool,

    /// Files to be processed
    #[arg()]
    files: Vec<String>,
}

// ---------------------------------------------------------------------------
// Hasher
// ---------------------------------------------------------------------------

const SNAIL_ITERATIONS: usize = 997usize;

enum Hasher {
    Default(SpongeHash256),
    Snailed(SpongeHash256<SNAIL_ITERATIONS>),
}

impl Hasher {
    pub fn new(info: &Option<String>, snail_mode: bool) -> Self {
        match info {
            Some(info) => match snail_mode {
                false => Self::Default(SpongeHash256::with_info(info)),
                true => Self::Snailed(SpongeHash256::with_info(info)),
            },
            None => match snail_mode {
                false => Self::Default(SpongeHash256::new()),
                true => Self::Snailed(SpongeHash256::new()),
            },
        }
    }

    pub fn update<T: AsRef<[u8]>>(&mut self, input: T) {
        match self {
            Hasher::Default(hasher) => hasher.update(input),
            Hasher::Snailed(hasher) => hasher.update(input),
        }
    }

    pub fn digest_to_slice(self, output: &mut [u8]) {
        match self {
            Hasher::Default(hasher) => hasher.digest_to_slice(output),
            Hasher::Snailed(hasher) => hasher.digest_to_slice(output),
        }
    }
}

// ---------------------------------------------------------------------------
// Process file
// ---------------------------------------------------------------------------

#[cfg(target_pointer_width = "64")]
const IO_BUFFER_SIZE: usize = 8192usize;
#[cfg(target_pointer_width = "32")]
const IO_BUFFER_SIZE: usize = 4096usize;
#[cfg(target_pointer_width = "16")]
const IO_BUFFER_SIZE: usize = 2048usize;

fn process_file(input: &mut impl Read, name: &str, size: usize, args: &Args) -> Result<(), Error> {
    let mut hasher = Hasher::new(&args.info, args.snail);
    let mut digest = [0u8; MAX_DIGEST_SIZE];
    let mut hexstr = [0u8; 2usize * MAX_DIGEST_SIZE];

    if !args.text {
        let mut buffer = [0u8; IO_BUFFER_SIZE];
        loop {
            match input.read(&mut buffer) {
                Ok(0) => break,
                Ok(length) => hasher.update(&buffer[..length]),
                Err(error) => return Err(error),
            }
        }
    } else {
        let mut lines = BufReader::new(input).lines();
        const LINE_BREAK: &str = "\n";
        if let Some(line) = lines.next() {
            hasher.update(&(line?));
            for line in lines {
                hasher.update(LINE_BREAK);
                hasher.update(&(line?));
            }
        }
    }

    hasher.digest_to_slice(&mut digest[..size]);
    encode_to_slice(&digest[..size], &mut hexstr[..(2usize * size)]).unwrap();

    if !args.plain {
        println!("{} {}", str::from_utf8(&hexstr).unwrap(), name);
    } else {
        println!("{}", str::from_utf8(&hexstr).unwrap());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Iterate input files
// ---------------------------------------------------------------------------

fn iterate_files(files: Iter<'_, String>, digest_size: usize, args: &Args) -> ExitCode {
    let mut errors: usize = 0usize;

    for file_name in files {
        match File::open(file_name) {
            Ok(mut file) => {
                if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                    if !args.quiet {
                        eprintln!("Is a directory: {:?}", file_name);
                    }
                    if !args.keep_going {
                        return ExitCode::FAILURE;
                    } else {
                        errors += 1usize;
                    }
                } else if let Err(error) = process_file(&mut file, file_name, digest_size, args) {
                    if !args.quiet {
                        eprintln!("Failed to read file: {:?} [{:?}]", file_name, error);
                    }
                    if !args.keep_going {
                        return ExitCode::FAILURE;
                    } else {
                        errors += 1usize;
                    }
                }
            }
            Err(error) => {
                if !args.quiet {
                    eprintln!("Failed to open input file: {:?} [{:?}]", file_name, error);
                }
                if !args.keep_going {
                    return ExitCode::FAILURE;
                } else {
                    errors += 1usize;
                }
            }
        }
    }

    if errors == 0usize { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn read_from_stdin(digest_size: usize, args: &Args) -> ExitCode {
    if let Err(error) = process_file(&mut stdin(), "-", digest_size, args) {
        if !args.quiet {
            eprintln!("Failed to read input data from 'stdin' stream: {:?}", error);
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    // Parse command-line args
    let args = Args::parse_from(wild::args_os());

    // Check for invalid combinations of options
    if args.text && args.binary {
        if !args.quiet {
            eprintln!("Error: Options '--binary' and '--text' are mutually exclusive!");
        }
        return ExitCode::FAILURE;
    }

    // Make sure that the digest size is divisble by eight
    if args.length.is_some_and(|value| value.get() % 8usize != 0usize) {
        if !args.quiet {
            eprintln!("Error: Digest output size must be divisble by eight! (given value: {})", args.length.unwrap().get());
        }
        return ExitCode::FAILURE;
    }

    // Compute and verify the digest size in bytes
    let digest_size = args.length.map(|value| value.get() / 8usize).unwrap_or(DEFAULT_DIGEST_SIZE);
    if digest_size > MAX_DIGEST_SIZE {
        if !args.quiet {
            eprintln!("Error: Digest output size exceeds the allowable maximum! (given value: {})", digest_size * 8usize);
        }
        return ExitCode::FAILURE;
    }

    // Process all files that were given on the command-line
    if !args.files.is_empty() {
        iterate_files(args.files.iter(), digest_size, &args)
    } else {
        read_from_stdin(digest_size, &args)
    }
}
