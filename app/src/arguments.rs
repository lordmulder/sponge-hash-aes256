// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use build_time::build_time_utc;
use clap::{ArgAction, ArgGroup, Error, Parser};
use const_format::formatcp;
use rustc_version_const::rustc_version_full;
use sponge_hash_aes256::version;
use std::{
    env::consts::{ARCH, OS},
    num::NonZeroUsize,
    path::PathBuf,
    sync::OnceLock,
};
use wild::args_os;

use crate::common::ExitStatus;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Build profile
const BUILD_PROFILE: &str = if cfg!(debug_assertions) { "debug" } else { "release" };

/// Version string
const VERSION: &str = formatcp!("v{} [SpongeHash-AES256 v{}] [{OS}] [{ARCH}] [{BUILD_PROFILE}]", env!("CARGO_PKG_VERSION"), version());

/// Full version string
const LONG_VERSION: &str = formatcp!("{VERSION}\nBuilt on: {}\nCompiled using rustc version: {}", build_time_utc!("%F, %T"), rustc_version_full());

/// Header line
const HEADER_LINE: &str = formatcp!("{} v{} (with SpongeHash-AES256 v{})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"), version());

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
#[command(group(ArgGroup::new("walk").args(["dirs", "recursive", "cross_dev"]).multiple(true)))]
pub struct Args {
    /// Read the input file(s) in binary mode, i.e., default mode
    #[arg(short, long, conflicts_with = "text")]
    pub binary: bool,

    /// Read the input file(s) in text mode
    #[arg(short, long, conflicts_with = "binary")]
    pub text: bool,

    /// Read and verify checksums from the provided input file(s)
    #[arg(short, long)]
    pub check: bool,

    /// Enable processing of directories as arguments
    #[arg(short, long, conflicts_with = "check")]
    pub dirs: bool,

    /// Recursively process the provided directories (implies -d)
    #[arg(short, long, conflicts_with = "check")]
    pub recursive: bool,

    /// Descend into directories on other devices (implies -r)
    #[arg(short = 'x', long, conflicts_with = "check")]
    pub cross_dev: bool,

    /// Iterate all kinds of files, instead of just regular files
    #[arg(short, long, requires = "walk")]
    pub all: bool,

    /// Continue processing even if errors are encountered.
    #[arg(short, long)]
    pub keep_going: bool,

    /// Digest output size, in bits (default: 256, maximum: 2048)
    #[arg(short, long, conflicts_with = "check")]
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
    #[arg(short, long, conflicts_with = "check")]
    pub plain: bool,

    /// Separate digest(s) by NULL characters instead of newlines
    #[arg(short = '0', long, alias = "zero", short_alias = 'z')]
    pub null: bool,

    /// Enable multi-threaded processing of input files
    #[arg(short, long, conflicts_with = "self_test")]
    pub multi_threading: bool,

    /// Explicitly flush 'stdout' stream after printing a digest
    #[arg(short, long)]
    pub flush: bool,

    /// Run the built-in self-test (BIST)
    #[arg(short = 'T', long, conflicts_with_all = ["check", "files"])]
    pub self_test: bool,

    /// Files to be processed
    #[arg()]
    pub files: Vec<PathBuf>,
}

/// Singleton instance
static ARGS_INSTANCE: OnceLock<Result<Args, Error>> = OnceLock::new();

/// Initialize command-line arguments
pub fn parse_command_line() -> Result<&'static Args, ExitStatus> {
    let instance = ARGS_INSTANCE.get_or_init(|| match Args::try_parse_from(args_os()) {
        Ok(mut args) => {
            args.recursive |= args.cross_dev;
            args.dirs |= args.recursive;
            Ok(args)
        }
        Err(error) => Err(error),
    });

    match instance {
        Ok(args) => Ok(args),
        Err(error) => {
            let _io = error.print();
            Err(ExitStatus::Failure)
        }
    }
}

/// Get header line
pub const fn header_line() -> &'static str {
    HEADER_LINE
}
