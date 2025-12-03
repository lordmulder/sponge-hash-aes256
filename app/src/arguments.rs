// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use build_time::build_time_utc;
use clap::{ArgAction, Parser};
use const_format::formatcp;
use rustc_version_const::rustc_version_full;
use sponge_hash_aes256::version;
use std::{
    env::consts::{ARCH, OS},
    num::NonZeroUsize,
    path::PathBuf,
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
#[command(version = VERSION)]
#[command(long_version = LONG_VERSION)]
pub struct Args {
    /// Read the input file(s) in binary mode, i.e., default mode
    #[arg(short, long)]
    pub binary: bool,

    /// Read the input file(s) in text mode
    #[arg(short, long)]
    pub text: bool,

    /// Read and verify checksums from the provided input file(s)
    #[arg(short, long)]
    pub check: bool,

    /// Enable processing of directories as arguments
    #[arg(short, long)]
    pub dirs: bool,

    /// Recursively process the provided directories (implies -d)
    #[arg(short, long)]
    pub recursive: bool,

    /// Continue processing even if errors are encountered.
    #[arg(short, long)]
    pub keep_going: bool,

    /// Digest output size, in bits (default: 256, maximum: 2048)
    #[arg(short, long)]
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
    #[arg(short, long)]
    pub plain: bool,

    /// Separate digest(s) by NULL characters instead of newlines
    #[arg(short = '0', long, alias = "zero", short_alias = 'z')]
    pub null: bool,

    /// Enable multi-threaded processing of input files
    #[arg(short, long)]
    pub multi_threading: bool,

    /// Explicitly flush 'stdout' stream after printing a digest
    #[arg(short, long)]
    pub flush: bool,

    /// Run the built-in self-test (BIST)
    #[arg(short = 'T', long)]
    pub self_test: bool,

    /// Files to be processed
    #[arg()]
    pub files: Vec<PathBuf>,
}

impl Args {
    pub fn parse_command_line() -> Self {
        Self::parse_from(args_os())
    }
}
