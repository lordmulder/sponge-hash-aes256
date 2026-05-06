// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

mod common;

use crate::common::{
    random::random_u64,
    utils::{digest_eq, get_file_name, run_binary, run_binary_and_exit, run_binary_to_file, run_binary_with_data, run_binary_with_env},
};

use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    ffi::{OsStr, OsString},
    fs::File,
    hint::black_box,
    io::{BufRead, BufReader, BufWriter, Write},
    iter,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[cfg(unix)]
use std::{
    fs::{set_permissions, Permissions},
    os::unix::fs::PermissionsExt,
};

#[cfg(unix)]
use crate::common::utils::{run_binary_from_file, run_binary_with_signal};

#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

#[cfg(target_os = "linux")]
use crate::common::utils::run_binary_unbuffered;

#[used]
static DROP_ROOT_CAPS: () = drop_root_caps::set_up();

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

// Input data to be hashed
static INPUT_MESSAGE: &[u8] = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

// Expected digest values
static EXPECTED: [&str; 47usize] = [
    "68c0656ee81830fd73031bd53af43c4a793a353c4e086ba27b9851206c17398d",
    "0d74c2e49bc2458915d78321ceddd9566bfee73b5bdf63ea0326cdbd78603afc",
    "a32cd2879cb337568324f064921072ce131d2ad981d84263731a3328c474187f",
    "06201c3a68c7b8812ef8c492c61d955972031273582a73f86085479173e11526",
    "d0075cf3e68f6c492d5f5527694ecb8ef4df9ca241c7eafd26d8c8435dfdef5b",
    "0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a8fbe372506a2add4",
    "bcfe521448677a659e765acc9d0ee4aa005531518a4279539e7793d2ba9c26db",
    "f555e89024343d383438be0a278a6ea7f8783335f9a795981fdd8a929f619616",
    "1d17d61280be5c77902f380608b1eea7327a90dadabb96b83122a1f27934fc9c",
    "f1792a0de1f8ed2747e9635a0f434e8aae920d4b5d17c3642a05f947c54a8876",
    "68c0656ee81830fd73031bd53af43c4a793a353c4e086ba27b9851206c17398d09387f3d5802f59869856d349b5e41b688ecf8b97358b414a18e3a946f011188",
    "68c0656ee81830fd73031bd53af43c4a793a353c4e086ba2",
    "0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a8fbe372506a2add4830545482c7142793861425a5e3d15811edf833008379b9ec2767aa204ae738d",
    "0f0309b4b5e00bbf5492efcb6a6fdfc890d5e1d695fd6f4a",
    "5525347e60fcf0c36a6939d2900388ca9562fb320b3d62fb82b5c496ada9010e",
    "690bfc8cce39ac385a3c16521d6f212a40e5447f358bb0d3cf7e282298b21ace",
    "cae29bda6c724974a27f45c7a993fe00c3f084f31d40d8491063adbec31f8594",
    "183b74b858525bd3827983cbce01ba6716adf98aa95d2723ef9f17f3a6db57f3",
    "27fab5a0bf44a2fe7e31a19eccc4ad17233498f20a488f6fa04781fc4bb45204",
    "38e0b15a62761581544bf4317babb2e89e1eb578a4b5b341267522c494554185",
    "d957410edab00e29dfc181cc941a067ec4105726bdf8a00bdeecc813ea860928",
    "8ba7c6f2a544bea08f90833ef8e9faaeb36dbbe516610fcee7295e81c709036c",
    "23f5fea30132ee640f1780516cc374c3042507c57cf540b5293cc50eac2fad4f",
    "19ee2d21b6bdb04feb8ac34c9036d502e1d43e1641d963e0c285c69bbda043da",
    "e5aaf13e4ac36cd35c331ed89c0c451b6d07163d1fabba9bbda9183486a55645",
    "34f3f9fd10aa864bd03631ed2142e29095210edf56ce8efd319154bab4a73eda",
    "1d0285d2ce3fa1ca745555daa301dc43569318f87d8d958d7aa841ba9b72c462",
    "6a7973490c36b46548aeeab412a3627eb896389662f366c5baf617e3b1446dbe",
    "cbca17a3893291669566f13ee10e20dcf58c8f77556048c6a5b0365ade0ff73c",
    "069047fb68beea4baf6f054ec0322621ae4e32cdeff3354d9b674318428db103",
    "e2755eb2eb3353b28249b1bb6d38390b0acb677a2e4cee1ade17802fc018ff4a",
    "e0c9b9ef955cb50354f5236ea5956cb30de1d6f78cc1d8b7586f277db177a4cc",
    "b38ee7d783ef9bd381003e96afce959829f64e1dbe6f5a3770f20080efc357ae",
    "aef5a52668dbe29365a1efe8863ece9d77058838e17449e80ed089947c1ae0b7",
    "a06aa9dc1c300910592b5f91dcaa77d51c88f495176a5f3bbce3c524e2c018c5",
    "568dec19bb459f51651caa5fa28a201e1c1557d817c1e6a4344b89c7d787c120",
    "ac412f4791b0823bc8e9527dfe70bbee3e1c1f4ad286c60184e263573451271b",
    "0fcb324f81264fde86df8b25df92b1f1c08051cc9b92414843c5044d90ff5759",
    "756349dfdfd63fb82bb4fa417b30c7695f86120f2a2d0c1dc2fa29a820c68442",
    "76d0b2b94003c069172a228866436925e43cda64e9a7f6f0c0bc92e9f282ef26",
    "658c2632fa26de9b7410bbe7a9a94b513875cc60ef499d2bd81f3aa159599c99",
    "3ccc937a835d8cd4af63007c741d75f1a55efcac8ef9da4503a7c0cf4f1cc05e",
    "a9f85f6c13049df99066ce72ca681ae0fa2d23cac7afff7da570c05638c856f2",
    "cfc9bca044ff820959a5fcd08d3096c2ef637e3fd68091118c83d9fc52e3e784",
    "2e6a8ce4c04f6ca518f06d109cb82514285b2e614584e2c65f874cf94ca074e5",
    "c75a794e49090b7a9a7144c0acb984e20f4534b4e11e5bbacbe2ec05d44fe85a",
    "3e948059e44ebe75efd4c4359853ecff5f337c96c23e9bc72f346eae8d05b8f2",
];

// Path to a non-existing file
#[cfg(windows)]
const FILE_PATH: &str = r#"C:\this\file\does\not\exist"#;
#[cfg(not(windows))]
const FILE_PATH: &str = "/this/file/does/not/exist";

// Path to a directory (not a file)
#[cfg(windows)]
const DIRECTORY_PATH: &str = r#"C:\Windows"#;
#[cfg(not(windows))]
const DIRECTORY_PATH: &str = "/usr";

// The standard input stream device file path
#[cfg(windows)]
const STDIN_DEV_FILE: &str = "CONIN$";
#[cfg(not(windows))]
const STDIN_DEV_FILE: &str = "/dev/stdin";

// ---------------------------------------------------------------------------
// Regular expressions
// ---------------------------------------------------------------------------

static REGEX_LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)\s([\x20-\x7E]+)$").unwrap());
static REGEX_PLAIN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([0-9a-fA-F]+)$").unwrap());
static REGEX_ZERO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([0-9a-fA-F]+)\s([\x20-\x7E]+)\x00").unwrap());
static REGEX_PLAIN_ZERO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([0-9a-fA-F]+)\x00").unwrap());
static REGEX_CHECK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^([\x20-\x7E]+):\s(\w+)$").unwrap());
static REGEX_CHECK_ZERO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\x20-\x7E]+):\s(\w+)\x00").unwrap());
static REGEX_VERSION: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^sponge256sum\s+v(\d+\.\d+\.\d+)[\s$]").unwrap());
static REGEX_HELP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^Usage:\s+sponge256sum(\.exe)?[\s$]").unwrap());
static REGEX_SELFTEST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^Successful.").unwrap());
static REGEX_UNKNOWN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"error: unexpected argument '([^']+)' found"#).unwrap());
static REGEX_MUTEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"error: the argument '([^']+)' cannot be used with '([^']+)'"#).unwrap());
static REGEX_MULTIPLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"error: the argument '([^']+)' cannot be used multiple times"#).unwrap());
static REGEX_MISSING_VAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"error: a value is required for '([^']+)' but none was supplied"#).unwrap());
static REGEX_MISSING_ARG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"error: the following required arguments were not provided:"#).unwrap());
static REGEX_INVALID_UTF: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"error: invalid UTF-8 was detected in one or more arguments"#).unwrap());
static REGEX_INVALID_VAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"error: invalid value '([^']+)' for '([^']+)':"#).unwrap());
static REGEX_LEN_DIV: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Error: Digest output size must be divisible by eight!").unwrap());
static REGEX_LEN_MAX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Error: Digest output size exceeds the allowable maximum!").unwrap());
static REGEX_INFO: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"Error: Length of context info must not exceed 255 characters!").unwrap());
static REGEX_FILE_NOENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Input file not found: "([^"]+)""#).unwrap());
static REGEX_FILE_FOPEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Failed to open input file: "([^"]+)""#).unwrap());
static REGEX_CHECK_NOENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Checksum file not found: "([^"]+)""#).unwrap());
static REGEX_CHECK_FOPEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Failed to open checksum file: "([^"]+)""#).unwrap());
static REGEX_MALFORMED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Malformed checksum file: "([^"]+)" \[line #(\d+)\]"#).unwrap());
static REGEX_TARGET_NOENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Target file not found: "([^"]+)"#).unwrap());
static REGEX_TARGET_FOPEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Failed to open target file: "([^"]+)"#).unwrap());
static REGEX_ENVIRON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Error: Value "([^"]+)" for environment variable "([^"]+)" is invalid!"#).unwrap());

#[cfg(unix)]
static REGEX_ABORTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)\bAborted: The process has been interrupted").unwrap());
#[cfg(unix)]
static REGEX_FILE_ISDIR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Input file is a directory: "([^"]+)""#).unwrap());
#[cfg(unix)]
static REGEX_CHECK_ISDIR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Checksum file is a directory: "([^"]+)""#).unwrap());
#[cfg(unix)]
static REGEX_TARGET_ISDIR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Target file is a directory: "([^"]+)"#).unwrap());
#[cfg(unix)]
static REGEX_STDIN_READ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Failed to read data from the standard input stream!"#).unwrap());

#[cfg(target_os = "linux")]
static REGEX_FILE_READ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Failed to read input file: "([^"]+)""#).unwrap());
#[cfg(target_os = "linux")]
static REGEX_TARGET_READ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Failed to read target file: "([^"]+)""#).unwrap());
#[cfg(target_os = "linux")]
static REGEX_CHECK_READ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Failed to read checksum file: "([^"]+)""#).unwrap());

// ---------------------------------------------------------------------------
// Miscellaneous functions
// ---------------------------------------------------------------------------

fn modify_checksum_file(original_file: &Path, modified_file: PathBuf, first_only: bool) -> PathBuf {
    fn modify_hex_char(character: &char) -> char {
        match *character {
            value if ('0'..='8').contains(&value) => char::from_u32(value as u32 + 1u32).unwrap(),
            '9' => 'a',
            value if ('a'..='e').contains(&value) => char::from_u32(value as u32 + 1u32).unwrap(),
            'f' => '0',
            _ => panic!("Invalid hex character: '{}'", *character),
        }
    }

    let reader = BufReader::new(File::open(original_file).unwrap());
    let mut first_line = true;
    let mut writer = BufWriter::new(File::create_new(&modified_file).unwrap());

    for line in reader.lines() {
        let mut line_modified: Vec<char> = line.unwrap().trim_start().chars().collect();
        if !line_modified.is_empty() {
            if first_line || (!first_only) {
                let first_char = line_modified.first_mut().unwrap();
                *first_char = modify_hex_char(first_char);
            }
            first_line = false;
            writeln!(&mut writer, "{}", String::from_iter(line_modified.into_iter())).unwrap();
        }
    }

    modified_file
}

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn do_test_file(expected: &str, file_name: &str, text_mode: bool, snail_level: usize, flush: bool) {
    let path = if !text_mode {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("binary").join(file_name)
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("text").join(file_name)
    };

    let mut parameters = Vec::with_capacity(7usize);

    if text_mode {
        parameters.push(OsStr::new("--text"));
    }

    for _ in 0..snail_level {
        parameters.push(OsStr::new("--snail"));
    }

    if flush {
        parameters.push(OsStr::new("--flush"));
    }

    parameters.push(path.as_os_str());

    let output = run_binary(parameters, true, false);
    let caps = REGEX_LINE.captures(&output).expect("Regex did not match!");

    assert!(digest_eq(caps.get(1).unwrap().as_str(), expected));
}

fn do_test_file_with_length(expected: &str, file_name: &str, length: u32) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("binary").join(file_name);
    let output = run_binary([OsStr::new("--length"), OsStr::new(&format!("{}", length)), path.as_os_str()], true, false);
    let caps = REGEX_LINE.captures(&output).expect("Regex did not match!");

    assert!(digest_eq(caps.get(1).unwrap().as_str(), expected));
}

fn do_test_file_with_info(expected: &str, file_name: &str, info: &str, snail_level: usize) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("binary").join(file_name);

    let mut parameters = Vec::with_capacity(6usize);
    parameters.extend_from_slice(&[OsStr::new("--info"), OsStr::new(info)]);

    for _ in 0..snail_level {
        parameters.push(OsStr::new("--snail"));
    }

    parameters.push(path.as_os_str());

    let output = run_binary(parameters.as_slice(), true, false);
    let caps = REGEX_LINE.captures(&output).expect("Regex did not match!");

    assert!(digest_eq(caps.get(1).unwrap().as_str(), expected));
}

fn do_test_multi_file(expected_map: &HashMap<&str, &str>, thread_count: NonZeroUsize) {
    let base_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let paths: Vec<PathBuf> = expected_map.values().map(|file_name| base_directory.join("tests").join("data").join("binary").join(file_name)).collect();

    let mut parameters = Vec::with_capacity(paths.len() + 1usize);
    let mut digest_set = HashSet::with_capacity(paths.len());

    if thread_count.get() > 1usize {
        parameters.push(OsStr::new("--multi-threading"));
    }

    paths.iter().for_each(|path| parameters.push(path.as_os_str()));

    let env = HashMap::from([("SPONGE256SUM_THREAD_COUNT", thread_count.to_string()), ("SPONGE256SUM_DIRWALK_STRATEGY", "BFS".to_owned())]);
    let output = run_binary_with_env(parameters, env, true, false);

    for caps in REGEX_LINE.captures_iter(&output) {
        let (digest, file_name) = (caps.get(1).unwrap().as_str(), get_file_name(caps.get(2).unwrap().as_str()));
        let expected_name = expected_map.get(digest).expect("Unknown digest!");
        assert!(digest_set.insert(digest));
        assert_eq!(file_name, *expected_name);
    }

    expected_map.keys().for_each(|digest| assert!(digest_set.contains(digest)));
}

fn do_test_dir(expected_map: &HashMap<&str, &str>, recursive: Option<bool>, multi_threading: bool, force_null: bool, force_plain: bool, no_color: bool) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("binary");
    let mut parameters = Vec::with_capacity(6usize);
    let mut digest_set = HashSet::with_capacity(expected_map.len());

    if let Some(cross_dev) = recursive {
        if cross_dev {
            parameters.push(OsStr::new("--cross-dev"));
        } else {
            parameters.push(OsStr::new("--recursive"));
        }
    } else {
        parameters.push(OsStr::new("--dirs"));
    }

    if multi_threading {
        parameters.push(OsStr::new("--multi-threading"));
    }

    if force_null {
        parameters.push(OsStr::new("--null"));
    }

    if force_plain {
        parameters.push(OsStr::new("--plain"));
    }

    if no_color {
        parameters.push(OsStr::new("--no-color"));
    }

    parameters.push(path.as_os_str());
    let output = run_binary(parameters.as_slice(), true, false);

    let matches = if force_null {
        if force_plain {
            REGEX_PLAIN_ZERO.captures_iter(&output)
        } else {
            REGEX_ZERO.captures_iter(&output)
        }
    } else if force_plain {
        REGEX_PLAIN.captures_iter(&output)
    } else {
        REGEX_LINE.captures_iter(&output)
    };

    for caps in matches {
        let digest = caps.get(1).unwrap().as_str();
        if !force_plain {
            let file_name = get_file_name(caps.get(2).unwrap().as_str());
            if !["LICENSE", "SHA512SUMS", "next"].iter().any(|str| file_name.eq_ignore_ascii_case(str)) {
                let expected_name = expected_map.get(digest).expect("Unknown digest!");
                assert!(digest_set.insert(digest));
                assert_eq!(file_name, *expected_name);
            }
        } else {
            assert!(digest_set.insert(digest)); /* no file name available */
        }
    }

    expected_map.keys().for_each(|digest| assert!(digest_set.contains(digest)));
}

fn do_test_data(expected: &str, data: &[u8], info: Option<&str>, snail_level: usize) {
    let mut parameters = Vec::with_capacity(6usize);

    if let Some(info) = info {
        parameters.extend_from_slice(&[OsStr::new("--info"), OsStr::new(info)]);
    }

    for _ in 0..snail_level {
        parameters.push(OsStr::new("--snail"));
    }

    let output = run_binary_with_data(parameters, data);
    let caps = REGEX_LINE.captures(&output).expect("Regex did not match!");

    assert!(digest_eq(caps.get(1).unwrap().as_str(), expected));
}

fn do_verify_files(modify: bool, file_count: usize, multi_threading: bool, force_null: bool, flush: bool) {
    let source_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data");
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));

    run_binary_to_file([OsStr::new("--recursive"), source_dir.as_os_str()], &check_file);

    let input_file = if modify {
        let modified_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("modified_{:016X}.txt", random_u64()));
        modify_checksum_file(&check_file, modified_file, false)
    } else {
        check_file.clone()
    };

    let mut parameters = Vec::with_capacity(6usize);
    parameters.extend_from_slice(&[OsStr::new("--check"), OsStr::new("--keep-going")]);

    if force_null {
        parameters.push(OsStr::new("--null"));
    }

    if multi_threading {
        parameters.push(OsStr::new("--multi-threading"));
    }

    if flush {
        parameters.push(OsStr::new("--flush"));
    }

    parameters.push(input_file.as_os_str());

    let mut result_set = HashSet::with_capacity(file_count);
    let output = run_binary(parameters, !modify, false);
    let matches = if force_null { REGEX_CHECK_ZERO.captures_iter(&output) } else { REGEX_CHECK.captures_iter(&output) };

    for caps in matches {
        let (file_name, result) = (caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str());
        if file_name.ends_with(".pdf") {
            assert_eq!(result, if modify { "FAILED" } else { "OK" });
            assert!(result_set.insert(file_name));
        }
    }

    assert_eq!(result_set.len(), file_count);
}

fn do_test_exit_code(files: &[&str], verify_mode: bool, modify: bool, keep_going: bool, expected_code: i32) {
    assert!(verify_mode || (!modify));
    let base_directory = Path::new(env!("CARGO_MANIFEST_DIR"));
    let paths: Vec<PathBuf> = files.iter().map(|file_name| base_directory.join("tests").join("data").join("binary").join(file_name)).collect();

    if !verify_mode {
        let mut parameters = Vec::with_capacity(paths.len() + 1usize);
        if keep_going {
            parameters.push(OsStr::new("--keep-going"));
        }
        paths.iter().for_each(|path| parameters.push(path.as_os_str()));
        assert_eq!(run_binary_and_exit(parameters), expected_code);
    } else {
        let parameters: Vec<&OsStr> = paths.iter().map(|path| path.as_os_str()).collect();
        let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
        run_binary_to_file(parameters, &check_file);

        let input_file = if modify {
            let modified_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
            modify_checksum_file(&check_file, modified_file, true)
        } else {
            check_file.clone()
        };

        let mut parameters = Vec::with_capacity(3usize);
        parameters.push(OsStr::new("--check"));
        if keep_going {
            parameters.push(OsStr::new("--keep-going"));
        }
        parameters.push(input_file.as_os_str());
        assert_eq!(run_binary_and_exit(parameters), expected_code);
    }
}

// ---------------------------------------------------------------------------
// Test cases
// ---------------------------------------------------------------------------

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// File tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_file_1a() {
    do_test_file(EXPECTED[0usize], "frank.pdf", false, 0usize, false);
}

#[test]
fn test_file_1b() {
    do_test_file(EXPECTED[1usize], "frank.pdf", false, 1usize, false);
}

#[test]
fn test_file_1c() {
    do_test_file(EXPECTED[2usize], "frank.pdf", false, 2usize, false);
}

#[test]
#[ignore]
fn test_file_1d() {
    do_test_file(EXPECTED[3usize], "frank.pdf", false, 3usize, false);
}

#[test]
#[ignore]
fn test_file_1e() {
    do_test_file(EXPECTED[4usize], "frank.pdf", false, 4usize, false);
}

#[test]
fn test_file_2a() {
    do_test_file(EXPECTED[5usize], "dracula.pdf", false, 0usize, false);
}

#[test]
fn test_file_2b() {
    do_test_file(EXPECTED[6usize], "dracula.pdf", false, 1usize, false);
}

#[test]
fn test_file_2c() {
    do_test_file(EXPECTED[7usize], "dracula.pdf", false, 2usize, false);
}

#[test]
#[ignore]
fn test_file_2d() {
    do_test_file(EXPECTED[8usize], "dracula.pdf", false, 3usize, false);
}

#[test]
#[ignore]
fn test_file_2e() {
    do_test_file(EXPECTED[9usize], "dracula.pdf", false, 4usize, false);
}

#[test]
fn test_file_3a() {
    do_test_file(EXPECTED[0usize], "frank.pdf", false, 0usize, true);
}

#[test]
fn test_file_3b() {
    do_test_file(EXPECTED[5usize], "dracula.pdf", false, 0usize, true);
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// File tests with info
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_file_with_len_1a() {
    do_test_file_with_length(EXPECTED[10usize], "frank.pdf", 512u32);
}

#[test]
fn test_file_with_len_1b() {
    do_test_file_with_length(EXPECTED[11usize], "frank.pdf", 192u32);
}

#[test]
fn test_file_with_len_2a() {
    do_test_file_with_length(EXPECTED[12usize], "dracula.pdf", 512u32);
}

#[test]
fn test_file_with_len_2b() {
    do_test_file_with_length(EXPECTED[13usize], "dracula.pdf", 192u32);
}

#[test]
fn test_file_with_info_1a() {
    do_test_file_with_info(EXPECTED[14usize], "frank.pdf", "whatchamacallit", 0usize);
}

#[test]
fn test_file_with_info_1b() {
    do_test_file_with_info(EXPECTED[15usize], "frank.pdf", "thingamabob", 0usize);
}

#[test]
fn test_file_with_info_1c() {
    do_test_file_with_info(EXPECTED[16usize], "frank.pdf", "whatchamacallit", 1usize);
}

#[test]
fn test_file_with_info_1d() {
    do_test_file_with_info(EXPECTED[17usize], "frank.pdf", "thingamabob", 2usize);
}

#[test]
#[ignore]
fn test_file_with_info_1e() {
    do_test_file_with_info(EXPECTED[18usize], "frank.pdf", "whatchamacallit", 3usize);
}

#[test]
#[ignore]
fn test_file_with_info_1f() {
    do_test_file_with_info(EXPECTED[19usize], "frank.pdf", "thingamabob", 4usize);
}

#[test]
fn test_file_with_info_2a() {
    do_test_file_with_info(EXPECTED[20usize], "dracula.pdf", "whatchamacallit", 0usize);
}

#[test]
fn test_file_with_info_2b() {
    do_test_file_with_info(EXPECTED[21usize], "dracula.pdf", "thingamabob", 0usize);
}

#[test]
fn test_file_with_info_2c() {
    do_test_file_with_info(EXPECTED[22usize], "dracula.pdf", "whatchamacallit", 1usize);
}

#[test]
fn test_file_with_info_2d() {
    do_test_file_with_info(EXPECTED[23usize], "dracula.pdf", "thingamabob", 2usize);
}

#[test]
#[ignore]
fn test_file_with_info_2e() {
    do_test_file_with_info(EXPECTED[24usize], "dracula.pdf", "whatchamacallit", 3usize);
}

#[test]
#[ignore]
fn test_file_with_info_2f() {
    do_test_file_with_info(EXPECTED[25usize], "dracula.pdf", "thingamabob", 4usize);
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Text file tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_text_file_1a() {
    do_test_file(EXPECTED[26usize], "alice29.txt", true, 0usize, false);
}

#[test]
fn test_text_file_1b() {
    do_test_file(EXPECTED[27usize], "alice29.txt", true, 1usize, false);
}

#[test]
fn test_text_file_1c() {
    do_test_file(EXPECTED[28usize], "alice29.txt", true, 2usize, false);
}

#[test]
#[ignore]
fn test_text_file_1d() {
    do_test_file(EXPECTED[29usize], "alice29.txt", true, 3usize, false);
}

#[test]
#[ignore]
fn test_text_file_1e() {
    do_test_file(EXPECTED[30usize], "alice29.txt", true, 4usize, false);
}

#[test]
fn test_text_file_2a() {
    do_test_file(EXPECTED[31usize], "asyoulik.txt", true, 0usize, false);
}

#[test]
fn test_text_file_2b() {
    do_test_file(EXPECTED[32usize], "asyoulik.txt", true, 1usize, false);
}

#[test]
fn test_text_file_2c() {
    do_test_file(EXPECTED[33usize], "asyoulik.txt", true, 2usize, false);
}

#[test]
#[ignore]
fn test_text_file_2d() {
    do_test_file(EXPECTED[34usize], "asyoulik.txt", true, 3usize, false);
}

#[test]
#[ignore]
fn test_text_file_2e() {
    do_test_file(EXPECTED[35usize], "asyoulik.txt", true, 4usize, false);
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Multi file tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_multi_file_1a() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_multi_file(&expected, NonZeroUsize::MIN);
}

#[test]
fn test_multi_file_1b() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_multi_file(&expected, NonZeroUsize::new(expected.len()).unwrap());
}

#[test]
fn test_multi_file_2a() {
    let expected = HashMap::from([(EXPECTED[5usize], "dracula.pdf"), (EXPECTED[0usize], "frank.pdf")]);
    do_test_multi_file(&expected, NonZeroUsize::MIN);
}

#[test]
fn test_multi_file_2b() {
    let expected = HashMap::from([(EXPECTED[5usize], "dracula.pdf"), (EXPECTED[0usize], "frank.pdf")]);
    do_test_multi_file(&expected, NonZeroUsize::new(expected.len()).unwrap());
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Directory tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_dir_1a() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, false, false, false, false);
}

#[test]
fn test_dir_1b() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, false, false, true, false);
}

#[test]
fn test_dir_1c() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, false, true, false, false);
}

#[test]
fn test_dir_1d() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, false, true, true, false);
}

#[test]
fn test_dir_1e() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, true, false, false, false);
}

#[test]
fn test_dir_1f() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, true, false, true, false);
}

#[test]
fn test_dir_1g() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, true, true, false, false);
}

#[test]
fn test_dir_1h() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, true, true, true, false);
}

#[test]
fn test_dir_1i() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, true, true, false, true);
}

#[test]
fn test_dir_1j() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf")]);
    do_test_dir(&expected, None, true, true, true, true);
}

#[test]
fn test_dir_2a() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), false, false, false, false);
}

#[test]
fn test_dir_2b() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), false, false, true, false);
}

#[test]
fn test_dir_2c() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), false, true, false, false);
}

#[test]
fn test_dir_2d() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), false, true, true, false);
}

#[test]
fn test_dir_2e() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), true, false, false, false);
}

#[test]
fn test_dir_2f() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), true, false, true, false);
}

#[test]
fn test_dir_2g() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), true, true, false, false);
}

#[test]
fn test_dir_2h() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), true, true, true, false);
}

#[test]
fn test_dir_2i() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), true, true, false, true);
}

#[test]
fn test_dir_2j() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(false), true, true, true, true);
}

#[test]
fn test_dir_3a() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), false, false, false, false);
}

#[test]
fn test_dir_3b() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), false, false, true, false);
}

#[test]
fn test_dir_3c() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), false, true, false, false);
}

#[test]
fn test_dir_3d() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), false, true, true, false);
}

#[test]
fn test_dir_3e() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), true, false, false, false);
}

#[test]
fn test_dir_3f() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), true, false, true, false);
}

#[test]
fn test_dir_3g() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), true, true, false, false);
}

#[test]
fn test_dir_3h() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), true, true, true, false);
}

#[test]
fn test_dir_3i() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), true, true, false, true);
}

#[test]
fn test_dir_3j() {
    let expected = HashMap::from([(EXPECTED[0usize], "frank.pdf"), (EXPECTED[5usize], "dracula.pdf"), (EXPECTED[36usize], "dorian.pdf")]);
    do_test_dir(&expected, Some(true), true, true, true, true);
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Data (stdin) tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_data_1a() {
    static STDIN_DATA: &[u8] = include_bytes!("data/binary/frank.pdf");
    do_test_data(EXPECTED[0usize], STDIN_DATA, None, 0usize);
}

#[test]
fn test_data_1b() {
    static STDIN_DATA: &[u8] = include_bytes!("data/binary/frank.pdf");
    do_test_data(EXPECTED[1usize], STDIN_DATA, None, 1usize);
}

#[test]
fn test_data_2a() {
    static STDIN_DATA: &[u8] = include_bytes!("data/binary/dracula.pdf");
    do_test_data(EXPECTED[5usize], STDIN_DATA, None, 0usize);
}

#[test]
fn test_data_2b() {
    static STDIN_DATA: &[u8] = include_bytes!("data/binary/dracula.pdf");
    do_test_data(EXPECTED[6usize], STDIN_DATA, None, 1usize);
}

#[test]
fn test_data_3a() {
    do_test_data(EXPECTED[37usize], INPUT_MESSAGE, None, 2usize);
}

#[test]
fn test_data_3b() {
    do_test_data(EXPECTED[38usize], INPUT_MESSAGE, None, 3usize);
}

#[test]
fn test_data_3c() {
    do_test_data(EXPECTED[39usize], INPUT_MESSAGE, None, 4usize);
}

#[test]
fn test_data_4a() {
    do_test_data(EXPECTED[40usize], INPUT_MESSAGE, Some("thingamabob"), 0usize);
}

#[test]
fn test_data_4b() {
    do_test_data(EXPECTED[41usize], INPUT_MESSAGE, Some("thingamabob"), 1usize);
}

#[test]
fn test_data_4c() {
    do_test_data(EXPECTED[42usize], INPUT_MESSAGE, Some("thingamabob"), 2usize);
}

#[test]
fn test_data_4d() {
    do_test_data(EXPECTED[43usize], INPUT_MESSAGE, Some("thingamabob"), 3usize);
}

#[test]
fn test_data_4e() {
    do_test_data(EXPECTED[44usize], INPUT_MESSAGE, Some("thingamabob"), 4usize);
}

#[test]
fn test_data_5a() {
    let output = run_binary_with_data([OsStr::new(STDIN_DEV_FILE)], INPUT_MESSAGE);
    let caps = REGEX_LINE.captures(&output).unwrap();
    assert!(digest_eq(caps.get(1).unwrap().as_str(), EXPECTED[45usize]));
}

#[test]
fn test_data_5b() {
    let output = run_binary_with_data([OsStr::new("--snail"), OsStr::new(STDIN_DEV_FILE)], INPUT_MESSAGE);
    let caps = REGEX_LINE.captures(&output).unwrap();
    assert!(digest_eq(caps.get(1).unwrap().as_str(), EXPECTED[46usize]));
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Verify tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_verify_1a() {
    do_verify_files(false, 3usize, false, false, false);
}

#[test]
fn test_verify_1b() {
    do_verify_files(false, 3usize, true, false, false);
}

#[test]
fn test_verify_2a() {
    do_verify_files(true, 3usize, false, false, false);
}

#[test]
fn test_verify_2b() {
    do_verify_files(true, 3usize, true, false, false);
}

#[test]
fn test_verify_3a() {
    do_verify_files(false, 3usize, false, true, false);
}

#[test]
fn test_verify_3b() {
    do_verify_files(false, 3usize, true, true, false);
}

#[test]
fn test_verify_4a() {
    do_verify_files(false, 3usize, false, false, true);
}

#[test]
fn test_verify_4b() {
    do_verify_files(true, 3usize, false, false, true);
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Exit code tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_exit_code_1a() {
    do_test_exit_code(&["frank.pdf", "dracula.pdf"], false, false, false, 0i32);
}

#[test]
fn test_exit_code_1b() {
    do_test_exit_code(&["frank.pdf", "dracula.pdf"], false, false, true, 0i32);
}

#[test]
fn test_exit_code_1c() {
    do_test_exit_code(&["fr0nk.pdf", "dracula.pdf"], false, false, false, 2i32);
}

#[test]
fn test_exit_code_1d() {
    do_test_exit_code(&["fr0nk.pdf", "dracula.pdf"], false, false, true, 1i32);
}

#[test]
fn test_exit_code_2a() {
    do_test_exit_code(&["frank.pdf", "dracula.pdf"], true, false, false, 0i32);
}

#[test]
fn test_exit_code_2b() {
    do_test_exit_code(&["frank.pdf", "dracula.pdf"], true, false, true, 0i32);
}

#[test]
fn test_exit_code_2c() {
    do_test_exit_code(&["frank.pdf", "dracula.pdf"], true, true, false, 2i32);
}

#[test]
fn test_exit_code_2d() {
    do_test_exit_code(&["frank.pdf", "dracula.pdf"], true, true, true, 1i32);
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Error tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(unix)]
#[test]
fn test_interrupt_1() {
    let output = run_binary_with_signal([OsStr::new("/dev/zero")], 3u64, 2i32, 3i32, true);
    assert!(REGEX_ABORTED.is_match(&output))
}

#[cfg(unix)]
#[test]
fn test_interrupt_2() {
    let output = run_binary_with_signal([OsStr::new("--multi-threading"), OsStr::new("/dev/zero")], 3u64, 2i32, 3i32, true);
    assert!(REGEX_ABORTED.is_match(&output))
}

#[test]
fn test_invalid_args_1a() {
    let output = run_binary([OsStr::new("-w")], false, true);
    assert!(REGEX_UNKNOWN.is_match(&output))
}
#[test]
fn test_invalid_args_1b() {
    let output = run_binary([OsStr::new("--foobar")], false, true);
    assert!(REGEX_UNKNOWN.is_match(&output))
}

#[test]
fn test_invalid_args_2a() {
    for (arg_1, arg_2) in [("--binary", "--text"), ("-b", "-t"), ("--quiet", "--no-color"), ("-q", "-n")] {
        let output = run_binary([OsStr::new(arg_1), OsStr::new(arg_2)], false, true);
        assert!(REGEX_MUTEX.is_match(&output))
    }
}

#[test]
fn test_invalid_args_2b() {
    for arg_2 in ["--dirs", "--recursive", "--cross-dev", "--plain"] {
        let output = run_binary([OsStr::new("--check"), OsStr::new(arg_2)], false, true);
        assert!(REGEX_MUTEX.is_match(&output))
    }
}

#[test]
fn test_invalid_args_2c() {
    let output = run_binary([OsStr::new("--check"), OsStr::new("--length"), OsStr::new("64")], false, true);
    assert!(REGEX_MUTEX.is_match(&output))
}

#[test]
fn test_invalid_args_2d() {
    for arg_2 in ["--multi-threading", "--check", "filename.txt"] {
        let output = run_binary([OsStr::new("--self-test"), OsStr::new(arg_2)], false, true);
        assert!(REGEX_MUTEX.is_match(&output))
    }
}

#[test]
fn test_invalid_args_2e() {
    for (arg_1, arg_2) in [("--binary", "--binary"), ("--binary", "-b"), ("-b", "-b"), ("--text", "--text"), ("--text", "-t"), ("-t", "-t")] {
        let output = run_binary([OsStr::new(arg_1), OsStr::new(arg_2)], false, true);
        assert!(REGEX_MULTIPLE.is_match(&output))
    }
}

#[test]
fn test_invalid_args_3a() {
    let output = run_binary([OsStr::new("--length")], false, true);
    assert!(REGEX_MISSING_VAL.is_match(&output))
}

#[test]
fn test_invalid_args_3b() {
    let output = run_binary([OsStr::new("--length"), OsStr::new("yikes")], false, true);
    assert!(REGEX_INVALID_VAL.is_match(&output))
}

#[test]
fn test_invalid_args_3c() {
    let output = run_binary([OsStr::new("--length"), OsStr::new("13")], false, true);
    assert!(REGEX_LEN_DIV.is_match(&output))
}

#[test]
fn test_invalid_args_3d() {
    let output = run_binary([OsStr::new("--length"), OsStr::new("8192")], false, true);
    assert!(REGEX_LEN_MAX.is_match(&output))
}

#[test]
fn test_invalid_args_4a() {
    let parameters: Vec<&OsStr> = iter::repeat_n(OsStr::new("--snail"), 5usize).collect();
    black_box(run_binary(parameters, false, true));
}

#[test]
fn test_invalid_args_4b() {
    let long_info = str::from_utf8(&[0x41u8; 256usize]).unwrap();
    let output = run_binary([OsStr::new("--info"), OsStr::new(long_info)], false, true);
    assert!(REGEX_INFO.is_match(&output))
}

#[test]
fn test_invalid_args_5a() {
    let output = run_binary([OsStr::new("--all")], false, true);
    assert!(REGEX_MISSING_ARG.is_match(&output))
}

#[test]
fn test_invalid_args_5b() {
    #[cfg(not(windows))]
    let invalid_string = unsafe { OsString::from_encoded_bytes_unchecked(b"\xE9".to_vec()) };
    #[cfg(windows)]
    let invalid_string = OsString::from_wide(&[0xD800]);
    let output = run_binary([OsStr::new("--info"), invalid_string.as_os_str()], false, true);
    assert!(REGEX_INVALID_UTF.is_match(&output))
}

#[test]
fn test_file_error_1a() {
    let output = run_binary([OsStr::new(FILE_PATH)], false, true);
    assert!(REGEX_FILE_NOENT.is_match(&output))
}

#[test]
fn test_file_error_1b() {
    let output = run_binary([OsStr::new("--multi-threading"), OsStr::new(FILE_PATH)], false, true);
    assert!(REGEX_FILE_NOENT.is_match(&output))
}

#[test]
fn test_file_error_2a() {
    let output = run_binary([OsStr::new(DIRECTORY_PATH)], false, true);
    #[cfg(windows)]
    assert!(REGEX_FILE_FOPEN.is_match(&output));
    #[cfg(unix)]
    assert!(REGEX_FILE_ISDIR.is_match(&output));
}

#[test]
fn test_file_error_2b() {
    let output = run_binary([OsStr::new("--multi-threading"), OsStr::new(DIRECTORY_PATH)], false, true);
    #[cfg(windows)]
    assert!(REGEX_FILE_FOPEN.is_match(&output));
    #[cfg(unix)]
    assert!(REGEX_FILE_ISDIR.is_match(&output));
}

#[cfg(unix)]
#[test]
fn test_file_error_3a() {
    let input_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("file_{:016X}.txt", random_u64()));
    File::create(&input_file).unwrap().write_all(b"justsomearbitrarydatainthefile\n").unwrap();
    set_permissions(&input_file, Permissions::from_mode(0o0u32)).unwrap();
    let output = run_binary([input_file.as_os_str()], false, true);
    assert!(REGEX_FILE_FOPEN.is_match(&output));
}

#[cfg(unix)]
#[test]
fn test_file_error_3b() {
    let input_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("file_{:016X}.txt", random_u64()));
    File::create(&input_file).unwrap().write_all(b"justsomearbitrarydatainthefile\n").unwrap();
    set_permissions(&input_file, Permissions::from_mode(0o0u32)).unwrap();
    let output = run_binary([OsStr::new("--multi-threading"), input_file.as_os_str()], false, true);
    assert!(REGEX_FILE_FOPEN.is_match(&output));
}

#[cfg(target_os = "linux")]
#[test]
fn test_file_error_4a() {
    let output = run_binary([OsStr::new("/proc/self/mem")], false, true);
    assert!(REGEX_FILE_READ.is_match(&output));
}

#[cfg(target_os = "linux")]
#[test]
fn test_file_error_4b() {
    let output = run_binary([OsStr::new("--multi-threading"), OsStr::new("/proc/self/mem")], false, true);
    assert!(REGEX_FILE_READ.is_match(&output));
}

#[cfg(unix)]
#[test]
fn test_file_error_5() {
    let output = run_binary_from_file::<[&OsStr; 0usize], _>([], Path::new("/"), false, true);
    assert!(REGEX_STDIN_READ.is_match(&output));
}

#[test]
fn test_check_error_1a() {
    let output = run_binary([OsStr::new("--check"), OsStr::new(FILE_PATH)], false, true);
    assert!(REGEX_CHECK_NOENT.is_match(&output))
}

#[test]
fn test_check_error_1b() {
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), OsStr::new(FILE_PATH)], false, true);
    assert!(REGEX_CHECK_NOENT.is_match(&output))
}

#[test]
fn test_check_error_2a() {
    let output = run_binary([OsStr::new("--check"), OsStr::new(DIRECTORY_PATH)], false, true);
    #[cfg(windows)]
    assert!(REGEX_CHECK_FOPEN.is_match(&output));
    #[cfg(unix)]
    assert!(REGEX_CHECK_ISDIR.is_match(&output));
}

#[test]
fn test_check_error_2b() {
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), OsStr::new(DIRECTORY_PATH)], false, true);
    #[cfg(windows)]
    assert!(REGEX_CHECK_FOPEN.is_match(&output));
    #[cfg(unix)]
    assert!(REGEX_CHECK_ISDIR.is_match(&output));
}

#[test]
fn test_check_error_3a() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"invalidchecksumfile\n").unwrap();
    let output = run_binary([OsStr::new("--check"), check_file.as_os_str()], false, true);
    assert!(REGEX_MALFORMED.is_match(&output))
}

#[test]
fn test_check_error_3b() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"invalidchecksumfile\n").unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), check_file.as_os_str()], false, true);
    assert!(REGEX_MALFORMED.is_match(&output))
}

#[test]
fn test_check_error_3c() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"invalidchecksumfile\n").unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--keep-going"), check_file.as_os_str()], false, true);
    assert!(REGEX_MALFORMED.is_match(&output))
}

#[test]
fn test_check_error_4a() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"00000000 this-file-does-not-exist\n").unwrap();
    let output = run_binary([OsStr::new("--check"), check_file.as_os_str()], false, true);
    assert!(REGEX_TARGET_NOENT.is_match(&output))
}

#[test]
fn test_check_error_4b() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"00000000 this-file-does-not-exist\n").unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), check_file.as_os_str()], false, true);
    assert!(REGEX_TARGET_NOENT.is_match(&output))
}

#[test]
fn test_check_error_4c() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"00000000 this-file-does-not-exist\n").unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--keep-going"), check_file.as_os_str()], false, true);
    assert!(REGEX_TARGET_NOENT.is_match(&output))
}

#[test]
fn test_check_error_5a() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(format!("00000000 {}\n", DIRECTORY_PATH).as_bytes()).unwrap();
    let output = run_binary([OsStr::new("--check"), check_file.as_os_str()], false, true);
    #[cfg(windows)]
    assert!(REGEX_TARGET_FOPEN.is_match(&output));
    #[cfg(unix)]
    assert!(REGEX_TARGET_ISDIR.is_match(&output));
}

#[test]
fn test_check_error_5b() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(format!("00000000 {}\n", DIRECTORY_PATH).as_bytes()).unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), check_file.as_os_str()], false, true);
    #[cfg(windows)]
    assert!(REGEX_TARGET_FOPEN.is_match(&output));
    #[cfg(unix)]
    assert!(REGEX_TARGET_ISDIR.is_match(&output));
}

#[cfg(unix)]
#[test]
fn test_check_error_6a() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("file_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"justsomearbitrarydatainthefile\n").unwrap();
    set_permissions(&check_file, Permissions::from_mode(0o0u32)).unwrap();
    let output = run_binary([OsStr::new("--check"), check_file.as_os_str()], false, true);
    assert!(REGEX_CHECK_FOPEN.is_match(&output))
}

#[cfg(unix)]
#[test]
fn test_check_error_6b() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("file_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"justsomearbitrarydatainthefile\n").unwrap();
    set_permissions(&check_file, Permissions::from_mode(0o0u32)).unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), check_file.as_os_str()], false, true);
    assert!(REGEX_CHECK_FOPEN.is_match(&output))
}

#[cfg(unix)]
#[test]
fn test_check_error_7a() {
    let target_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("file_{:016X}.txt", random_u64()));
    File::create(&target_file).unwrap().write_all(b"justsomearbitrarydatainthefile\n").unwrap();
    set_permissions(&target_file, Permissions::from_mode(0o0u32)).unwrap();
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(format!("00000000 {}\n", target_file.as_os_str().to_string_lossy()).as_bytes()).unwrap();
    let output = run_binary([OsStr::new("--check"), check_file.as_os_str()], false, true);
    assert!(REGEX_TARGET_FOPEN.is_match(&output));
}

#[cfg(unix)]
#[test]
fn test_check_error_7b() {
    let target_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("file_{:016X}.txt", random_u64()));
    File::create(&target_file).unwrap().write_all(b"justsomearbitrarydatainthefile\n").unwrap();
    set_permissions(&target_file, Permissions::from_mode(0o0u32)).unwrap();
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(format!("00000000 {}\n", target_file.as_os_str().to_string_lossy()).as_bytes()).unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), check_file.as_os_str()], false, true);
    assert!(REGEX_TARGET_FOPEN.is_match(&output));
}

#[cfg(target_os = "linux")]
#[test]
fn test_check_error_8a() {
    let output = run_binary([OsStr::new("--check"), OsStr::new("/proc/self/mem")], false, true);
    assert!(REGEX_CHECK_READ.is_match(&output))
}

#[cfg(target_os = "linux")]
#[test]
fn test_check_error_8b() {
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), OsStr::new("/proc/self/mem")], false, true);
    assert!(REGEX_CHECK_READ.is_match(&output))
}

#[cfg(target_os = "linux")]
#[test]
fn test_check_error_8c() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"00000000 /proc/self/mem\n").unwrap();
    let output = run_binary([OsStr::new("--check"), check_file.as_os_str()], false, true);
    assert!(REGEX_TARGET_READ.is_match(&output))
}

#[cfg(target_os = "linux")]
#[test]
fn test_check_error_8d() {
    let check_file = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("checksums_{:016X}.txt", random_u64()));
    File::create(&check_file).unwrap().write_all(b"00000000 /proc/self/mem\n").unwrap();
    let output = run_binary([OsStr::new("--check"), OsStr::new("--multi-threading"), check_file.as_os_str()], false, true);
    assert!(REGEX_TARGET_READ.is_match(&output))
}

#[test]
fn test_invalid_env_1a() {
    let env = HashMap::from([("SPONGE256SUM_DIRWALK_STRATEGY", "foo".to_owned())]);
    let output = run_binary_with_env([""; 0usize], env, false, true);
    assert!(REGEX_ENVIRON.is_match(&output))
}

#[test]
fn test_invalid_env_1b() {
    let env = HashMap::from([("SPONGE256SUM_DIRWALK_STRATEGY", "1".to_owned())]);
    let output = run_binary_with_env([""; 0usize], env, false, true);
    assert!(REGEX_ENVIRON.is_match(&output))
}

#[test]
fn test_invalid_env_2a() {
    let env = HashMap::from([("SPONGE256SUM_THREAD_COUNT", "foo".to_owned())]);
    let output = run_binary_with_env([""; 0usize], env, false, true);
    assert!(REGEX_ENVIRON.is_match(&output))
}

#[test]
fn test_invalid_env_2b() {
    let env = HashMap::from([("SPONGE256SUM_THREAD_COUNT", "-1".to_owned())]);
    let output = run_binary_with_env([""; 0usize], env, false, true);
    assert!(REGEX_ENVIRON.is_match(&output))
}

#[test]
fn test_invalid_env_3a() {
    let env = HashMap::from([("SPONGE256SUM_SELFTEST_PASSES", "foo".to_owned())]);
    let output = run_binary_with_env([""; 0usize], env, false, true);
    assert!(REGEX_ENVIRON.is_match(&output))
}

#[test]
fn test_invalid_env_3b() {
    let env = HashMap::from([("SPONGE256SUM_SELFTEST_PASSES", "0".to_owned())]);
    let output = run_binary_with_env([""; 0usize], env, false, true);
    assert!(REGEX_ENVIRON.is_match(&output))
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Version and help tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
fn test_version() {
    let output = run_binary([OsStr::new("--version")], true, false);
    let caps = REGEX_VERSION.captures(&output).expect("Regex did not match!");
    assert_eq!(caps.get(1).unwrap().as_str(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_help() {
    assert!(REGEX_HELP.is_match(&run_binary([OsStr::new("--help")], true, false)));
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ANSI color tests
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn test_unbuffered_1() {
    let output = run_binary_unbuffered([OsStr::new(FILE_PATH)], false);
    assert!(output.trim_ascii_start().starts_with("\u{1b}[1;31m[sponge256sum]\u{1b}[22;31m Input file not found:"));
}

#[cfg(target_os = "linux")]
#[test]
#[ignore]
fn test_unbuffered_2() {
    let output = run_binary_unbuffered([OsStr::new("--no-color"), OsStr::new(FILE_PATH)], false);
    assert!(output.trim_ascii_start().starts_with("[sponge256sum] Input file not found:"));
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Self-test
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[test]
#[ignore]
fn test_selftest() {
    let env = HashMap::from([("SPONGE256SUM_SELFTEST_PASSES", 1.to_string())]);
    assert!(REGEX_SELFTEST.is_match(&run_binary_with_env([OsStr::new("--self-test")], env, true, false)));
}
