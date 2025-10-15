// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
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
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)\s[\x20-\x7E]+$").unwrap());

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join(file_name);
    let output = if snail_mode { run_binary([OsStr::new("--snail"), path.as_os_str()]) } else { run_binary([path.as_os_str()]) };
    let caps = REGEX.captures(&output).expect("Regex did not match!");

    assert_eq!(caps.get(1).unwrap().as_str(), expected);
}

fn do_test_dir(expected_map: &HashMap<&str, &str>, recursive: bool, force_null: bool) {
    static REGEX_1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)\s([\x20-\x7E]+)$").unwrap());
    static REGEX_2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([0-9a-fA-F]+)\s([\x20-\x7E]+)\x00").unwrap());

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data");
    let mut digest_set = HashSet::with_capacity(expected_map.len());

    let output = if recursive {
        if force_null {
            run_binary([OsStr::new("--recursive"), OsStr::new("--null"), path.as_os_str()])
        } else {
            run_binary([OsStr::new("--recursive"), path.as_os_str()])
        }
    } else {
        if force_null {
            run_binary([OsStr::new("--dirs"), OsStr::new("--null"), path.as_os_str()])
        } else {
            run_binary([OsStr::new("--dirs"), path.as_os_str()])
        }
    };

    for caps in (if force_null { &REGEX_2 } else { &REGEX_1 }).captures_iter(&output) {
        let digest = caps.get(1).unwrap().as_str();
        let file_name = caps.get(2).unwrap().as_str().split(|c| c == '/' || c == '\\').last().expect("No file name!");
        if !(file_name.eq("LICENSE") || file_name.eq("SHA512SUMS")) {
            let expected_name = expected_map.get(digest).expect("Unknownd digest!");
            assert!(digest_set.insert(digest));
            assert_eq!(file_name, *expected_name);
        }
    }

    for file_name in expected_map.keys() {
        assert!(digest_set.contains(file_name));
    }
}

fn do_test_file_with_length(expected: &str, file_name: &str, length: u32) {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)\s[\x20-\x7E]+$").unwrap());

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join(file_name);
    let output = run_binary([OsStr::new("--length"), OsStr::new(&format!("{}", length)), path.as_os_str()]);
    let caps = REGEX.captures(&output).expect("Regex did not match!");

    assert_eq!(caps.get(1).unwrap().as_str(), expected);
}

fn do_test_file_with_info(expected: &str, file_name: &str, info: &str) {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)\s[\x20-\x7E]+$").unwrap());

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join(file_name);
    let output = run_binary([OsStr::new("--info"), OsStr::new(info), path.as_os_str()]);
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

const EXPECTED_1: &str = "68c0656ee81830fd73031bd53af43c4a793a353c4e086ba27b9851206c17398d";
const EXPECTED_2: &str = "0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a8fbe372506a2add4";
const EXPECTED_3: &str = "ac412f4791b0823bc8e9527dfe70bbee3e1c1f4ad286c60184e263573451271b";
const EXPECTED_4: &str = "0d74c2e49bc2458915d78321ceddd9566bfee73b5bdf63ea0326cdbd78603afc";
const EXPECTED_5: &str = "bcfe521448677a659e765acc9d0ee4aa005531518a4279539e7793d2ba9c26db";
const EXPECTED_6: &str =
    "68c0656ee81830fd73031bd53af43c4a793a353c4e086ba27b9851206c17398d09387f3d5802f59869856d349b5e41b688ecf8b97358b414a18e3a946f011188";
const EXPECTED_7: &str =
    "0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a8fbe372506a2add4830545482c7142793861425a5e3d15811edf833008379b9ec2767aa204ae738d";
const EXPECTED_8: &str = "5525347e60fcf0c36a6939d2900388ca9562fb320b3d62fb82b5c496ada9010e";
const EXPECTED_9: &str = "d957410edab00e29dfc181cc941a067ec4105726bdf8a00bdeecc813ea860928";

#[test]
fn test_file_1a() {
    do_test_file(EXPECTED_1, "frank.pdf", false);
}

#[test]
fn test_file_1b() {
    do_test_file(EXPECTED_4, "frank.pdf", true);
}

#[test]
fn test_file_1c() {
    do_test_file_with_length(EXPECTED_6, "frank.pdf", 512u32);
}

#[test]
fn test_file_1d() {
    do_test_file_with_info(EXPECTED_8, "frank.pdf", "whatchamacallit");
}

#[test]
fn test_file_2a() {
    do_test_file(EXPECTED_2, "dracula.pdf", false);
}

#[test]
fn test_file_2b() {
    do_test_file(EXPECTED_5, "dracula.pdf", true);
}

#[test]
fn test_file_2c() {
    do_test_file_with_length(EXPECTED_7, "dracula.pdf", 512u32);
}

#[test]
fn test_file_2d() {
    do_test_file_with_info(EXPECTED_9, "dracula.pdf", "whatchamacallit");
}

#[test]
fn test_dir_1a() {
    let mut expected = HashMap::with_capacity(2);
    expected.insert(EXPECTED_1, "frank.pdf");
    expected.insert(EXPECTED_2, "dracula.pdf");
    do_test_dir(&expected, false, false);
}

#[test]
fn test_dir_1b() {
    let mut expected = HashMap::with_capacity(2);
    expected.insert(EXPECTED_1, "frank.pdf");
    expected.insert(EXPECTED_2, "dracula.pdf");
    do_test_dir(&expected, false, true);
}

#[test]
fn test_dir_2a() {
    let mut expected = HashMap::with_capacity(2);
    expected.insert(EXPECTED_1, "frank.pdf");
    expected.insert(EXPECTED_2, "dracula.pdf");
    expected.insert(EXPECTED_3, "dorian.pdf");
    do_test_dir(&expected, true, false);
}

#[test]
fn test_dir_2b() {
    let mut expected = HashMap::with_capacity(2);
    expected.insert(EXPECTED_1, "frank.pdf");
    expected.insert(EXPECTED_2, "dracula.pdf");
    expected.insert(EXPECTED_3, "dorian.pdf");
    do_test_dir(&expected, true, true);
}

#[test]
fn test_data_1a() {
    static STDIN_DATA: &[u8] = include_bytes!("data/frank.pdf");
    do_test_data(EXPECTED_1, STDIN_DATA, false);
}

#[test]
fn test_data_1b() {
    static STDIN_DATA: &[u8] = include_bytes!("data/frank.pdf");
    do_test_data(EXPECTED_4, STDIN_DATA, true);
}

#[test]
fn test_data_2a() {
    static STDIN_DATA: &[u8] = include_bytes!("data/dracula.pdf");
    do_test_data(EXPECTED_2, STDIN_DATA, false);
}

#[test]
fn test_data_2b() {
    static STDIN_DATA: &[u8] = include_bytes!("data/dracula.pdf");
    do_test_data(EXPECTED_5, STDIN_DATA, true);
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
