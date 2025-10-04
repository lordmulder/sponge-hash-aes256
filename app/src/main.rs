// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

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
//!   -k, --keep-going       Keep going, even if an input file can not be read
//!   -l, --length <LENGTH>  Digest output size, in bits (default: 256, maximum: 1024)
//!   -i, --info <INFO>      Include additional context information
//!   -s, --snail...         Enable "snail" mode, i.e., slow down the hash computation
//!   -q, --quiet            Do not output any error messages or warnings
//!   -p, --plain            Print digest(s) in plain format, i.e., without file names
//!   -f, --flush            Explicitely flush 'stdout' stream after printing a digest
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
//! ## License
//!
//! Copyright (C) 2025 by LoRd_MuldeR &lt;mulder2@gmx.de&gt;
//!
//! Permission to use, copy, modify, and/or distribute this software for any purpose with or without fee is hereby granted.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

#![doc(hidden)]
#![doc(html_no_source)]

use clap::{ArgAction, Parser, command};
use const_format::formatcp;
use ctrlc::set_handler;
use hex::encode_to_slice;
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, PKG_VERSION as LIB_VERSION, SpongeHash256};
use std::{
    env::consts::{ARCH, OS},
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader, Error as IoError, Read, Write, stdin, stdout},
    num::NonZeroUsize,
    process::ExitCode,
    slice::Iter,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// Maximum allowable digest size, specified in bytes
const MAX_DIGEST_SIZE: usize = 128usize;

// Type definition
type Flag = Arc<AtomicBool>;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

enum Error {
    Io(IoError),
    Aborted,
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::Io(error)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Aborted => write!(f, "Interrupted by user!"),
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Build profile
#[cfg(not(debug_assertions))]
const BUILD_PROFILE: &str = "release";
#[cfg(debug_assertions)]
const BUILD_PROFILE: &str = "debug";

/// Header line
const HEADER_LINE: &str = formatcp!("{} v{} (with SpongeHash-AES256 v{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), LIB_VERSION);

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
    #[arg(short, long, action = ArgAction::Count)]
    snail: u8,

    /// Do not output any error messages or warnings
    #[arg(short, long)]
    quiet: bool,

    /// Print digest(s) in plain format, i.e., without file names
    #[arg(short, long)]
    plain: bool,

    /// Explicitely flush 'stdout' stream after printing a digest
    #[arg(short, long)]
    flush: bool,

    /// Files to be processed
    #[arg()]
    files: Vec<String>,
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

/// Conditional printing of error message
macro_rules! print_error {
    ($args:ident, $($message:tt)*) => {
        if !$args.quiet {
            eprintln!($($message)*);
        }
    };
}

/// Unified error handling routine
macro_rules! handle_error {
    ($args:ident, $err_counter:ident, $($message:tt)*) => {{
        print_error!($args, $($message)*);
        if $args.keep_going {
            $err_counter += 1usize;
        } else {
            return ExitCode::FAILURE;
        }
    }};
}

// ---------------------------------------------------------------------------
// Hasher
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
    pub fn new(info: &Option<String>, snail_mode: u8) -> Self {
        match info {
            Some(info) => match snail_mode {
                0u8 => Self::Default(SpongeHash256::with_info(info)),
                1u8 => Self::SnailV1(SpongeHash256::with_info(info)),
                2u8 => Self::SnailV2(SpongeHash256::with_info(info)),
                3u8 => Self::SnailV3(SpongeHash256::with_info(info)),
                4u8 => Self::SnailV4(SpongeHash256::with_info(info)),
                _ => unreachable!(),
            },
            None => match snail_mode {
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
// Print digest
// ---------------------------------------------------------------------------

fn print_digest(output: &mut impl Write, digest: &[u8], name: &str, size: usize, args: &Args) -> Result<(), Error> {
    let mut hexstr = [0u8; 2usize * MAX_DIGEST_SIZE];
    encode_to_slice(&digest[..size], &mut hexstr[..(2usize * size)]).unwrap();

    if args.plain {
        writeln!(output, "{}", str::from_utf8(&hexstr[..(2usize * size)]).unwrap())?;
    } else {
        writeln!(output, "{} {}", str::from_utf8(&hexstr[..(2usize * size)]).unwrap(), name)?;
    }

    if args.flush {
        output.flush()?;
    }

    Ok(())
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

/// Check whether the process has been interrupted
macro_rules! check_running {
    ($flag:ident) => {
        if !$flag.load(Ordering::Relaxed) {
            return Err(Error::Aborted);
        }
    };
}

/// Process a single input file
fn process_file(input: &mut impl Read, output: &mut impl Write, name: &str, size: usize, args: &Args, running: &Flag) -> Result<(), Error> {
    let mut digest = [0u8; MAX_DIGEST_SIZE];
    let mut hasher = Hasher::new(&args.info, args.snail);

    if !args.text {
        let mut buffer = [0u8; IO_BUFFER_SIZE];
        loop {
            check_running!(running);
            match input.read(&mut buffer)? {
                0 => break,
                length => hasher.update(&buffer[..length]),
            }
        }
    } else {
        let mut lines = BufReader::new(input).lines();
        const LINE_BREAK: &str = "\n";
        if let Some(line) = lines.next() {
            hasher.update(&(line?));
            for line in lines {
                check_running!(running);
                hasher.update(LINE_BREAK);
                hasher.update(&(line?));
            }
        }
    }

    hasher.digest_to_slice(&mut digest[..size]);
    print_digest(output, &digest, name, size, args)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Iterate input files
// ---------------------------------------------------------------------------

/// Iterate a list of input files
fn iterate_files(files: Iter<'_, String>, digest_size: usize, args: &Args, running: Flag) -> ExitCode {
    let mut output = stdout().lock();
    let mut err_counter: usize = 0usize;

    for file_name in files {
        match File::open(file_name) {
            Ok(mut file) => {
                if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                    handle_error!(args, err_counter, "Input is a directory: {:?}", file_name);
                } else {
                    match process_file(&mut file, &mut output, file_name, digest_size, args, &running) {
                        Ok(_) => {}
                        Err(Error::Aborted) => {
                            print_error!(args, "Aborted: The process has been interrupted by the user!");
                            return ExitCode::FAILURE;
                        }
                        Err(error) => handle_error!(args, err_counter, "Failed to read file: {:?} [{:?}]", file_name, error),
                    }
                }
            }
            Err(error) => handle_error!(args, err_counter, "Failed to open input file: {:?} [{:?}]", file_name, error),
        }
    }

    if err_counter == 0usize { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// Read data from the `stdin` stream
fn read_from_stdin(digest_size: usize, args: &Args, running: Flag) -> ExitCode {
    let mut input = stdin().lock();
    let mut output = stdout().lock();

    match process_file(&mut input, &mut output, "-", digest_size, args, &running) {
        Ok(_) => ExitCode::SUCCESS,
        Err(Error::Aborted) => {
            print_error!(args, "Aborted: The process has been interrupted by the user!");
            ExitCode::FAILURE
        }
        Err(error) => {
            print_error!(args, "Failed to read input data from 'stdin' stream: {:?}", error);
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const MAX_SNAIL_COUNT: u8 = 4u8;

/// Applicationm entry point (“main” function)
fn main() -> ExitCode {
    // Parse command-line args
    let args = Args::parse_from(wild::args_os());

    // Check for incompatible arguments
    if args.text && args.binary {
        print_error!(args, "Error: Options '--binary' and '--text' are mutually exclusive!");
        return ExitCode::FAILURE;
    }

    // Check for too many snail options passed
    if args.snail > MAX_SNAIL_COUNT {
        print_error!(args, "Error: Options '--snail' must not be set more than four times!");
        return ExitCode::FAILURE;
    }

    // Make sure that the digest size is divisble by eight
    if args.length.is_some_and(|value| value.get() % 8usize != 0usize) {
        print_error!(args, "Error: Digest output size must be divisble by eight! (given value: {})", args.length.unwrap().get());
        return ExitCode::FAILURE;
    }

    // Check the maximum allowed info string length
    if args.info.as_ref().is_some_and(|str| str.len() > u8::MAX as usize) {
        print_error!(args, "Error: Length of \"info\" must not exceed 255 characters! (given length: {})", args.info.unwrap().len());
        return ExitCode::FAILURE;
    }

    // Compute and verify the digest size in bytes
    let digest_size = args.length.map(|value| value.get() / 8usize).unwrap_or(DEFAULT_DIGEST_SIZE);
    if digest_size > MAX_DIGEST_SIZE {
        print_error!(args, "Error: Digest output size exceeds the allowable maximum! (given value: {})", digest_size * 8usize);
        return ExitCode::FAILURE;
    }

    // Install the interrupt handler
    let running = Arc::new(AtomicBool::new(true));
    let flag = running.clone();
    set_handler(move || flag.store(false, Ordering::SeqCst)).expect("Failed to register CTRL+C handler!");

    // Process all files that were given on the command-line
    if !args.files.is_empty() {
        iterate_files(args.files.iter(), digest_size, &args, running)
    } else {
        read_from_stdin(digest_size, &args, running)
    }
}
