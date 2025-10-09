// SPDX-License-Identifier: 0BSD
// sponge256sum
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
//!   -d, --dirs             Enable processing of directories as arguments
//!   -r, --recursive        Recursively process the provided directories (implies -d)
//!   -k, --keep-going       Continue processing even if errors are encountered
//!   -l, --length <LENGTH>  Digest output size, in bits (default: 256, maximum: 1024)
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
use hex_literal::hex;
use rand_pcg::{
    Pcg64,
    rand_core::{RngCore, SeedableRng},
};
use sponge_hash_aes256::{DEFAULT_DIGEST_SIZE, PKG_VERSION as LIB_VERSION, SpongeHash256};
use std::{
    env::consts::{ARCH, OS},
    fs::DirEntry,
    sync::mpsc::{self, Receiver},
};
use std::{
    ffi::OsStr,
    fmt::Debug,
    fs::{File, metadata, read_dir},
    io::{BufRead, BufReader, Error as IoError, Read, Write, stdin, stdout},
    num::NonZeroUsize,
    path::PathBuf,
    process::ExitCode,
    slice::Iter,
    str::from_utf8,
    time::Instant,
};

/// Maximum allowable digest size, specified in bytes
const MAX_DIGEST_SIZE: usize = 128usize;

// Type definition
type Flag = Receiver<bool>;

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
#[command(after_help = "If no input files are specified, reads input data from the 'stdin' stream.\n\
    Returns a non-zero exit code if any errors occurred; otherwise, zero.")]
#[command(version = VERSION_STR)]
#[command(before_help = HEADER_LINE)]
struct Args {
    /// Read the input file(s) in binary mode, i.e., default mode
    #[arg(short, long)]
    binary: bool,

    /// Read the input file(s) in text mode
    #[arg(short, long)]
    text: bool,

    /// Enable processing of directories as arguments
    #[arg(short, long)]
    dirs: bool,

    /// Recursively process the provided directories (implies -d)
    #[arg(short, long)]
    recursive: bool,

    /// Continue processing even if errors are encountered.
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

    /// Separate digest(s) by NULL characters instead of newlines
    #[arg(short = '0', long)]
    null: bool,

    /// Explicitely flush 'stdout' stream after printing a digest
    #[arg(short, long)]
    flush: bool,

    /// Run the built-in self-test (BIST)
    #[arg(short = 'T', long)]
    self_test: bool,

    /// Files to be processed
    #[arg()]
    files: Vec<PathBuf>,
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
            *$err_counter += 1usize;
        } else {
            return false;
        }
    }};
}

/// Check whether the process has been interrupted
macro_rules! check_running {
    ($channel:ident) => {
        if $channel.try_recv().unwrap_or_default() {
            return Err(Error::Aborted);
        }
    };
    ($args:ident, $channel:ident) => {
        if $channel.try_recv().unwrap_or_default() {
            print_error!($args, "Aborted: The process has been interrupted by the user!");
            return false;
        }
    };
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

fn print_digest(output: &mut impl Write, digest: &[u8], name: &OsStr, size: usize, args: &Args) -> Result<(), Error> {
    let mut hexstr = [0u8; 2usize * MAX_DIGEST_SIZE];
    encode_to_slice(&digest[..size], &mut hexstr[..(2usize * size)]).unwrap();

    if args.null {
        if args.plain {
            write!(output, "{}\0", from_utf8(&hexstr[..(2usize * size)]).unwrap())?;
        } else {
            write!(output, "{}\0{}\0", from_utf8(&hexstr[..(2usize * size)]).unwrap(), name.to_string_lossy())?;
        }
    } else if args.plain {
        writeln!(output, "{}", from_utf8(&hexstr[..(2usize * size)]).unwrap())?;
    } else {
        writeln!(output, "{} {}", from_utf8(&hexstr[..(2usize * size)]).unwrap(), name.to_string_lossy())?;
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

/// Process a single input file
fn process_file(input: &mut impl Read, output: &mut impl Write, name: &OsStr, size: usize, args: &Args, running: &Flag) -> Result<(), Error> {
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
// Read input file or stream
// ---------------------------------------------------------------------------

/// Read data from a file
fn read_file(path: &PathBuf, output: &mut impl Write, digest_size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    match File::open(path) {
        Ok(mut file) => {
            if file.metadata().is_ok_and(|meta| meta.is_dir()) {
                handle_error!(args, errors, "Input is a directory: {:?}", path);
            } else {
                match process_file(&mut file, output, path.as_os_str(), digest_size, args, running) {
                    Ok(_) => {}
                    Err(Error::Aborted) => {
                        print_error!(args, "Aborted: The process has been interrupted by the user!");
                        return false;
                    }
                    Err(error) => handle_error!(args, errors, "Failed to read file: {:?} [{:?}]", path, error),
                }
            }
        }
        Err(error) => handle_error!(args, errors, "Failed to open input file: {:?} [{:?}]", path, error),
    }

    true
}

/// Read data from the `stdin` stream
fn read_from_stdin(output: &mut impl Write, digest_size: usize, args: &Args, running: Flag) -> bool {
    let mut input = stdin().lock();

    match process_file(&mut input, output, OsStr::new("-"), digest_size, args, &running) {
        Ok(_) => true,
        Err(Error::Aborted) => {
            print_error!(args, "Aborted: The process has been interrupted by the user!");
            false
        }
        Err(error) => {
            print_error!(args, "Failed to read input data from 'stdin' stream: {:?}", error);
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Iterate input files/directories
// ---------------------------------------------------------------------------

/// Iterate a list of input files
fn iterate_files(files: Iter<'_, PathBuf>, output: &mut impl Write, digest_size: usize, args: &Args, running: Flag) -> bool {
    let mut errors = 0usize;
    let handle_dirs = args.dirs || args.recursive;

    for file_name in files {
        check_running!(args, running);
        if handle_dirs && metadata(file_name).is_ok_and(|meta| meta.is_dir()) {
            if !iterate_directory(file_name, output, digest_size, args, &running, &mut errors) {
                return false;
            }
        } else if !read_file(file_name, output, digest_size, args, &running, &mut errors) {
            return false;
        }
    }

    if args.keep_going && (errors > 0usize) {
        print_error!(args, "Warning: {} file(s) were skipped due to errors.", errors);
    }

    errors == 0usize
}

/// Iterate all files and sub-directories in a directory
fn iterate_directory(path: &PathBuf, output: &mut impl Write, digest_size: usize, args: &Args, running: &Flag, errors: &mut usize) -> bool {
    match read_dir(path) {
        Ok(dir_iter) => {
            for element in dir_iter {
                check_running!(args, running);
                match element {
                    Ok(dir_entry) => {
                        if is_directory(&dir_entry) {
                            if args.recursive && (!iterate_directory(&dir_entry.path(), output, digest_size, args, running, errors)) {
                                return false;
                            }
                        } else if !read_file(&dir_entry.path(), output, digest_size, args, running, errors) {
                            return false;
                        }
                    }
                    Err(error) => {
                        handle_error!(args, errors, "Failed to read directory: {:?} [{:?}]", path, error);
                    }
                }
            }
        }
        Err(error) => {
            handle_error!(args, errors, "Failed to open directory: {:?} [{:?}]", path, error);
        }
    }

    true
}

/// Check if directory entry is a directory or a symlink to a directory
fn is_directory(entry: &DirEntry) -> bool {
    match entry.metadata() {
        Ok(meta) => {
            let file_type = meta.file_type();
            match file_type.is_dir() {
                true => true,
                false => match file_type.is_symlink() {
                    true => metadata(entry.path()).is_ok_and(|info| info.is_dir()),
                    false => false,
                },
            }
        }
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Self-test
// ---------------------------------------------------------------------------

const PCG64_SEEDVALUE: u64 = 18446744073709551557u64;
const DIGEST_EXPECTED: [u8; DEFAULT_DIGEST_SIZE] = hex!("1f4232bab771e668edb99834527d1f632d32963164e0ca25e218210942f41947");

fn arrays_equal<const N: usize>(array0: &[u8; N], array1: &[u8; N]) -> bool {
    let mut mask = 0u8;
    for (value0, value1) in array0.iter().zip(array1.iter()) {
        mask |= value0 ^ value1;
    }
    mask == 0u8
}

fn self_test(output: &mut impl Write, args: &Args, running: Flag) -> bool {
    let _ = writeln!(output, "{}\n", HEADER_LINE);
    let _ = writeln!(output, "Self-test is running, please be patient...");
    let _ = output.flush();

    let start_time = Instant::now();

    let mut source = Pcg64::seed_from_u64(PCG64_SEEDVALUE);
    let mut buffer = [0u8; 4093usize];
    let mut hasher = SpongeHash256::default();

    for _ in 0u32..524287u32 {
        source.fill_bytes(&mut buffer);
        hasher.update(buffer);
        check_running!(args, running);
    }

    let digest_computed = hasher.digest();
    let elapsed = start_time.elapsed().as_secs_f64();

    if arrays_equal(&digest_computed, &DIGEST_EXPECTED) {
        let _ = writeln!(output, "Successful.\n");
        let _ = writeln!(output, "Test completed successfully in {:.1} seconds.", elapsed);
        true
    } else {
        let _ = writeln!(output, "Failure !!!\n");
        let _ = writeln!(output, "Digest computed: {:02x?}", &digest_computed[..]);
        let _ = writeln!(output, "Digest expected: {:02x?}", &DIGEST_EXPECTED[..]);
        false
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
        print_error!(args, "\n{}", include_str!("../../.assets/text/goat.txt"));
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
    let (stop_tx, stop_rx) = mpsc::channel();
    set_handler(move || stop_tx.send(true).expect("Failed to send the 'stop' flag!")).expect("Failed to register CTRL+C handler!");

    // Acquire stdout handle
    let mut output = stdout().lock();

    // Run built-in self-test, if it was requested by the user
    let success = if args.self_test {
        self_test(&mut output, &args, stop_rx)
    } else {
        // Process all files and directories that were given on the command-line
        if args.files.is_empty() {
            read_from_stdin(&mut output, digest_size, &args, stop_rx)
        } else {
            iterate_files(args.files.iter(), &mut output, digest_size, &args, stop_rx)
        }
    };

    if success { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
