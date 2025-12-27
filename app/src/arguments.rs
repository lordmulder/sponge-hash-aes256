// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use build_time::build_time_utc;
use clap::{
    error::{ContextKind, ContextValue, Error, ErrorKind},
    ArgAction, Parser,
};
use const_format::formatcp;
use rustc_version_const::rustc_version_full;
use sponge_hash_aes256::version;
use std::{
    env::consts::{ARCH, OS},
    num::NonZeroUsize,
    path::PathBuf,
    process::ExitCode,
};
use wild::args_os;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Build profile
const BUILD_PROFILE: &str = if cfg!(debug_assertions) { "debug" } else { "release" };

/// Version string
pub const VERSION: &str = formatcp!("v{} [SpongeHash-AES256 v{}] [{OS}] [{ARCH}] [{BUILD_PROFILE}]", env!("CARGO_PKG_VERSION"), version());

/// Full version string
pub const LONG_VERSION: &str = formatcp!("{VERSION}\nBuilt on: {}\nCompiled using rustc version: {}", build_time_utc!("%F, %T"), rustc_version_full());

/// Header line
pub const HEADER_LINE: &str = formatcp!("{} v{} (with SpongeHash-AES256 v{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), version());

/// About text
const ABOUT_TEXT: &str = "A sponge-based secure hash function that uses AES-256 as its internal PRF.\n\
    This software is released under the Zero-Clause BSD License.";

/// Additional help text
const HELP_TEXT: &str = "If no input files are specified, reads input data from the 'stdin' stream.\n\
    Returns a non-zero exit code if any errors occurred; otherwise, zero.\n\
    For details please refer to: <https://crates.io/crates/sponge-hash-aes256>";

// ---------------------------------------------------------------------------
// Command-line arguments
// ---------------------------------------------------------------------------

/// SpongeHash-AES256 command-line tool
#[derive(Parser, Debug, Clone)]
#[command(about = ABOUT_TEXT)]
#[command(after_help = HELP_TEXT)]
#[command(before_help = HEADER_LINE)]
#[command(long_version = LONG_VERSION)]
#[command(version = VERSION)]
pub struct Args {
    /// Read the input file(s) in binary mode, i.e., default mode
    #[arg(short, long)]
    #[arg(group = "input_mode")]
    pub binary: bool,

    /// Read the input file(s) in text mode
    #[arg(short, long)]
    #[arg(group = "input_mode")]
    pub text: bool,

    /// Read and verify checksums from the provided input file(s)
    #[arg(short, long, group = "mtx_dirs", group = "mtx_recursive", group = "mtx_all", group = "mtx_length", group = "mtx_plain", group = "mtx_selftest")]
    pub check: bool,

    /// Enable processing of directories as arguments
    #[arg(short, long, group = "mtx_dirs")]
    pub dirs: bool,

    /// Recursively process the provided directories (implies -d)
    #[arg(short, long, group = "mtx_recursive")]
    pub recursive: bool,

    /// Iterate all kinds of files, instead of just regular files
    #[arg(short, long, group = "mtx_all")]
    pub all: bool,

    /// Continue processing even if errors are encountered.
    #[arg(short, long)]
    pub keep_going: bool,

    /// Digest output size, in bits (default: 256, maximum: 2048)
    #[arg(short, long, group = "mtx_length")]
    pub length: Option<NonZeroUsize>,

    /// Include additional context information
    #[arg(short, long)]
    pub info: Option<String>,

    /// Enable "snail" mode, i.e., slow down the hash computation
    #[arg(short, long, action = ArgAction::Count)]
    pub snail: u8,

    /// Do not output any error messages or warnings
    #[arg(short, long)]
    pub quiet: bool,

    /// Print digest(s) in plain format, i.e., without file names
    #[arg(short, long, group = "mtx_plain")]
    pub plain: bool,

    /// Separate digest(s) by NULL characters instead of newlines
    #[arg(short = '0', long, alias = "zero", short_alias = 'z')]
    pub null: bool,

    /// Enable multi-threaded processing of input files
    #[arg(short, long, group = "mtx_threads")]
    pub multi_threading: bool,

    /// Explicitly flush 'stdout' stream after printing a digest
    #[arg(short, long)]
    pub flush: bool,

    /// Run the built-in self-test (BIST)
    #[arg(short = 'T', long, group = "mtx_selftest", group = "mtx_threads")]
    pub self_test: bool,

    /// Files to be processed
    #[arg()]
    pub files: Vec<PathBuf>,
}

impl Args {
    /// Parse command-line arguments
    pub fn try_parse_command_line() -> Result<Self, ExitCode> {
        match Self::try_parse_from(args_os()) {
            Ok(args) => Ok(args),
            Err(error) => Err(print_arg_error(error)),
        }
    }
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

macro_rules! print_arg_error {
    ($fmt:literal $(,$arg:expr)*$(,)?) => {
        eprintln!(concat!("[sponge256sum] Error: ", $fmt) $(, $arg)*)
    };
}

#[inline]
fn context_str(error: &Error, kind: ContextKind) -> &str {
    static EMPTY_STRING: String = String::new();
    if let Some(ContextValue::String(str_value)) = error.get(kind) {
        str_value
    } else {
        &EMPTY_STRING
    }
}

/// Print argument parser error
fn print_arg_error(error: Error) -> ExitCode {
    match error.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
            eprint!("{}", error);
            ExitCode::SUCCESS
        }
        ErrorKind::UnknownArgument => {
            print_arg_error!("Unknown option \"{}\" encountered!", context_str(&error, ContextKind::InvalidArg));
            ExitCode::FAILURE
        }
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => {
            let (invalid_arg, invalid_value) = (context_str(&error, ContextKind::InvalidArg), context_str(&error, ContextKind::InvalidValue));
            if invalid_value.is_empty() {
                print_arg_error!("The required value for option \"{}\" is missing!", invalid_arg);
            } else {
                print_arg_error!("The given value \"{}\" for option \"{}\" is invalid!", invalid_value, invalid_arg);
            }
            ExitCode::FAILURE
        }
        ErrorKind::ArgumentConflict => {
            let (invalid_arg, prior_arg) = (context_str(&error, ContextKind::InvalidArg), context_str(&error, ContextKind::PriorArg));
            if prior_arg.is_empty() || (prior_arg == invalid_arg) {
                print_arg_error!("The option \"{}\" can not be used more than once!", invalid_arg);
            } else {
                print_arg_error!("The options \"{}\" and \"{}\" are mutually exclusive!", invalid_arg, prior_arg);
            }
            ExitCode::FAILURE
        }
        other => {
            print_arg_error!("Invalid command-line arguments! ({:?})", other);
            ExitCode::FAILURE
        }
    }
}
