// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

use hex::decode_to_slice;
use num::Integer;
use sponge_hash_aes256::DEFAULT_DIGEST_SIZE;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use tinyvec::TinyVec;

#[cfg(unix)]
use std::{thread, time::Duration};

#[cfg(unix)]
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};

#[cfg(target_os = "linux")]
use which::which;

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

pub fn run_binary<I, S>(args: I, expected_success: bool, force_stderr: bool) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(if force_stderr { Stdio::null() } else { Stdio::piped() })
        .stderr(if force_stderr { Stdio::piped() } else { Stdio::null() })
        .stdin(Stdio::null())
        .output()
        .expect("Failed to run binary!");

    assert_eq!(output.status.success(), expected_success);
    String::from_utf8(if force_stderr { output.stderr } else { output.stdout }).unwrap()
}

pub fn run_binary_with_data<I, S>(args: I, data: &[u8]) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to run binary!");

    child.stdin.as_ref().unwrap().write_all(data).expect("Failed to write data!");
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

pub fn run_binary_to_file<I, S>(args: I, dest_file: &Path)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let dest_file = File::create_new(dest_file).expect("Failed to create output file!");
    let child = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(Stdio::from(dest_file))
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("Failed to run binary!");

    assert!(child.wait_with_output().unwrap().status.success());
}

pub fn run_binary_with_env<I, S>(args: I, env: HashMap<&str, String>, expected_success: bool, force_stderr: bool) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(if force_stderr { Stdio::null() } else { Stdio::piped() })
        .stderr(if force_stderr { Stdio::piped() } else { Stdio::null() })
        .stdin(Stdio::null())
        .envs(env)
        .output()
        .expect("Failed to run binary!");

    assert_eq!(output.status.success(), expected_success);
    String::from_utf8(if force_stderr { output.stderr } else { output.stdout }).unwrap()
}

#[cfg(unix)]
pub fn run_binary_with_signal<I, S>(args: I, delay: u64, signal: i32, expected_status: i32, force_stderr: bool) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(if force_stderr { Stdio::null() } else { Stdio::piped() })
        .stderr(if force_stderr { Stdio::piped() } else { Stdio::null() })
        .stdin(Stdio::null())
        .spawn()
        .expect("Failed to run binary!");

    thread::sleep(Duration::from_secs(delay));
    kill(Pid::from_raw(child.id() as i32), Signal::try_from(signal).unwrap()).expect("Failed to send signal!");

    let output = child.wait_with_output().expect("Failed to wait for process!");
    assert_eq!(output.status.code().unwrap_or(-1i32), expected_status);
    String::from_utf8(if force_stderr { output.stderr } else { output.stdout }).unwrap()
}

pub fn run_binary_and_exit<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("Failed to run binary!");

    let exit_status = child.wait().expect("Failed to wait for process!");
    exit_status.code().expect("Failed to get exit code!")
}

#[cfg(target_os = "linux")]
pub fn run_binary_unbuffered<I, S>(args: I, expected_success: bool) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let unbuffer_exe = which("expect_unbuffer").expect("The command 'expect_unbuffer' could not be found!");

    let output = Command::new(unbuffer_exe)
        .arg(env!("CARGO_BIN_EXE_sponge256sum"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .output()
        .expect("Failed to run binary!");

    assert_eq!(output.status.success(), expected_success);
    String::from_utf8(output.stdout).unwrap()
}

pub fn get_file_name(path: &str) -> &str {
    path.split(['/', '\\']).next_back().unwrap_or(path)
}

pub fn decode_digest(digest_hex: &str) -> Option<TinyVec<[u8; DEFAULT_DIGEST_SIZE]>> {
    let (length, remainder) = digest_hex.len().div_rem(&2usize);
    if (remainder != 0usize) || (length < 1usize) {
        return None;
    }

    let mut decoded: TinyVec<[u8; DEFAULT_DIGEST_SIZE]> = TinyVec::with_capacity(length);
    decoded.resize(length, 0u8);
    decode_to_slice(digest_hex, &mut decoded).ok().map(|_| decoded)
}

pub fn digest_eq(hexstr_1: &str, hexstr_2: &str) -> bool {
    if let (Some(digest_1), Some(digest_2)) = (decode_digest(hexstr_1), decode_digest(hexstr_2)) {
        if digest_1.len() == digest_2.len() {
            let mut diff_mask = 0u8;
            for (byte_1, byte_2) in digest_1.iter().zip(digest_2.iter()) {
                diff_mask |= byte_1 ^ byte_2;
            }
            return diff_mask == 0u8;
        }
    }

    false /* mismatch! */
}
