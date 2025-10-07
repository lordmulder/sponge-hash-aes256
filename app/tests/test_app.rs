// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use regex::Regex;
use std::{
    ffi::OsStr,
    io::Write,
    iter,
    path::Path,
    process::{Command, Stdio},
    sync::LazyLock,
};

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn run_binary<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .stdin(Stdio::null())
        .output()
        .expect("Failed to run binary!");

    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn run_binary_with_data<I, S>(args: I, data: &[u8]) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to run binary!");

    child.stdin.as_ref().unwrap().write_all(data).expect("Failed to write data!");
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

fn do_test_file(expected: &str, file_name: &str, snail_mode: bool) {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)[\s$]").unwrap());

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join(file_name);
    let output = if snail_mode { run_binary([OsStr::new("--snail"), path.as_os_str()]) } else { run_binary([path.as_os_str()]) };
    let caps = REGEX.captures(&output).expect("Regex did not match!");

    assert_eq!(caps.get(1).unwrap().as_str(), expected);
}

fn do_test_file_with_length(expected: &str, file_name: &str, length: u32) {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)[\s$]").unwrap());

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join(file_name);
    let output = run_binary([OsStr::new("--length"), OsStr::new(&format!("{}", length)), path.as_os_str()]);
    let caps = REGEX.captures(&output).expect("Regex did not match!");

    assert_eq!(caps.get(1).unwrap().as_str(), expected);
}

fn do_test_data(expected: &str, data: &[u8], snail_mode: bool) {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]{64})\s+-").unwrap());
    const NO_ARGS: iter::Empty<&OsStr> = iter::empty::<&OsStr>();

    let output = if snail_mode { run_binary_with_data([OsStr::new("--snail")], data) } else { run_binary_with_data(NO_ARGS, data) };
    let caps = REGEX.captures(&output).expect("Regex did not match!");

    assert_eq!(caps.get(1).unwrap().as_str(), expected);
}

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

#[test]
fn test_file_1a() {
    do_test_file("68c0656ee81830fd73031bd53af43c4a793a353c4e086ba27b9851206c17398d", "frank.pdf", false);
}

#[test]
fn test_file_1b() {
    do_test_file("0d74c2e49bc2458915d78321ceddd9566bfee73b5bdf63ea0326cdbd78603afc", "frank.pdf", true);
}

#[test]
fn test_file_1c() {
    do_test_file_with_length(
        "68c0656ee81830fd73031bd53af43c4a793a353c4e086ba27b9851206c17398d09387f3d5802f59869856d349b5e41b688ecf8b97358b414a18e3a946f011188",
        "frank.pdf",
        512u32,
    );
}

#[test]
fn test_file_2a() {
    do_test_file("0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a8fbe372506a2add4", "dracula.pdf", false);
}

#[test]
fn test_file_2b() {
    do_test_file("bcfe521448677a659e765acc9d0ee4aa005531518a4279539e7793d2ba9c26db", "dracula.pdf", true);
}

#[test]
fn test_file_2c() {
    do_test_file_with_length(
        "0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a8fbe372506a2add4830545482c7142793861425a5e3d15811edf833008379b9ec2767aa204ae738d",
        "dracula.pdf",
        512u32,
    );
}

#[test]
fn test_data_1a() {
    static STDIN_DATA: &[u8] = include_bytes!("data/frank.pdf");
    do_test_data("68c0656ee81830fd73031bd53af43c4a793a353c4e086ba27b9851206c17398d", STDIN_DATA, false);
}

#[test]
fn test_data_1b() {
    static STDIN_DATA: &[u8] = include_bytes!("data/frank.pdf");
    do_test_data("0d74c2e49bc2458915d78321ceddd9566bfee73b5bdf63ea0326cdbd78603afc", STDIN_DATA, true);
}

#[test]
fn test_data_2a() {
    static STDIN_DATA: &[u8] = include_bytes!("data/dracula.pdf");
    do_test_data("0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a8fbe372506a2add4", STDIN_DATA, false);
}

#[test]
fn test_data_2b() {
    static STDIN_DATA: &[u8] = include_bytes!("data/dracula.pdf");
    do_test_data("bcfe521448677a659e765acc9d0ee4aa005531518a4279539e7793d2ba9c26db", STDIN_DATA, true);
}

#[test]
fn test_version() {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^sponge256sum\s+v(\d+\.\d+\.\d+)[\s$]").unwrap());

    let output = run_binary([OsStr::new("--version")]);
    let caps = REGEX.captures(&output).expect("Regex did not match!");

    assert_eq!(caps.get(1).unwrap().as_str(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_help() {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^Usage:\s+sponge256sum(\.exe)?[\s$]").unwrap());
    assert!(REGEX.is_match(&run_binary([OsStr::new("--help")])));
}

#[test]
#[ignore]
fn test_selftest() {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^Successful.").unwrap());
    assert!(REGEX.is_match(&run_binary([OsStr::new("--self-test")])));
}
