// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#[test]
pub fn test_case_1a() {
    do_test(&hex!("ad7706c5683877d3dc6d502ecd5da678527bb8dfa5607970383bd8295b4f2366"), None, "");
}

#[test]
pub fn test_case_1b() {
    do_test(&hex!("4cc8f52921f0a0e761f5c3de136ee81e100131eb69e02341423cc81d27fb08ce"), Some("thingamajig"), "");
}

#[test]
pub fn test_case_2a() {
    do_test(
        &hex!("af2e6095d2e7e484504672353b3dd0723649a19ac73aa51359ec9f84ab941e53"),
        None,
        "abc",
    );
}

#[test]
pub fn test_case_2b() {
    do_test(
        &hex!("054f1284424eada25482864a9de5eef19f63d607301ce3474755b52d446e4e48"),
        Some("thingamajig"),
        "abc",
    );
}

#[test]
pub fn test_case_3a() {
    do_test(
        &hex!("831e84c5b2a07ccd54ece77d5f8a5d072437bfb7307684af5377855ab445be2e"),
        None,
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    );
}

#[test]
pub fn test_case_3b() {
    do_test(
        &hex!("2e38fdc04d38f9f78c06be1c6c173a6966359a7d45c6d6228350713c36763065"),
        Some("thingamajig"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    );
}

#[test]
pub fn test_case_4a() {
    do_test(
        &hex!("fd9a5d163425b2c1fd15bc4df0964ea4f995ccbcdae0c09f6e8f9b579d9216a8"),
        None,
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_4b() {
    do_test(
        &hex!("005036c74f39a40b811db6a36649c55da46600a99338b53a81fad4b6a9bb2787"),
        Some("thingamajig"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_5a() {
    do_test(
        &hex!("6029dc9de5f733b219d8ba5431ab6f904085754dc47f64887067899dc4819fd3"),
        None,
        str::from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    );
}

#[test]
pub fn test_case_5b() {
    do_test(
        &hex!("e7185c8e582ce95250a54be4439a056e784c1ff12d2df5b886f7765917aadc0f"),
        Some("thingamajig"),
        str::from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    );
}
