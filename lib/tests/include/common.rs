// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#[test]
pub fn test_case_1a() {
    do_test(&hex!("1c6f2074b336cec62f12dea2f6ab14814e9a8d9395f1a2c807724aa30a52df53"), None, "");
}

#[test]
pub fn test_case_1b() {
    do_test(&hex!("31ff6e7b23abadee453e11f5c2a4499e4f11803b50f22ddb8c0a393999927617"), Some("thingamajig"), "");
}

#[test]
pub fn test_case_2a() {
    do_test(
        &hex!("9dc26aca9b2a55d059ee357777ad0d3f4deafd4d86acc324bb52a258895e6767"),
        None,
        "abc",
    );
}

#[test]
pub fn test_case_2b() {
    do_test(
        &hex!("d93562a5a751a2047dbb959f5330822a2181d5a14c23548a7479f59fea319090"),
        Some("thingamajig"),
        "abc",
    );
}

#[test]
pub fn test_case_3a() {
    do_test(
        &hex!("601fb5758166b85f834a8ac923043891478f3c528c944b05edca28de3d5f4979"),
        None,
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    );
}

#[test]
pub fn test_case_3b() {
    do_test(
        &hex!("649118caf5c0be584dcace3ed0e2831ee6cc76485603aa54eeef07e3f90438d1"),
        Some("thingamajig"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    );
}

#[test]
pub fn test_case_4a() {
    do_test(
        &hex!("5b1982a036812f51d7e0795fe832bb54c94ca467bec1d901bb1ebdb86b0cd979"),
        None,
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_4b() {
    do_test(
        &hex!("ea4c4ec93686a175f16a6d20c6f978eb00e12a52d7c5f36cd6dbbe2a7eb7b7b9"),
        Some("thingamajig"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_5a() {
    do_test(
        &hex!("8e5c46c7e1c1e1409513d185bf167c53d87f9aab82494e5483b0db6b3bddb227"),
        None,
        str::from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    );
}

#[test]
pub fn test_case_5b() {
    do_test(
        &hex!("09c92ac46a6eb44af0f969aca30b79cc6e37cd8f67d604baa5b1db87e43cfd63"),
        Some("thingamajig"),
        str::from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    );
}
