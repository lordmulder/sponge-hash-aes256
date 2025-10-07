// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#[test]
pub fn test_case_1a() {
    do_test(&hex!("af46c9b65f45e2a1bd7025e1b108a76ec349aab7485fc6892f83717161dfc40f"), None, "");
}

#[test]
pub fn test_case_1b() {
    do_test(&hex!("c26e1a9ada9d9112f5374c5d7e44de04fa3cd6f60e6d1b7b4df875e30004b39b"), Some("thingamajig"), "");
}

#[test]
pub fn test_case_2a() {
    do_test(
        &hex!("5ba80675dc5567c83fba8720951b71658a0d9ca9fc28eabc48cc133349d241c9"),
        None,
        "abc",
    );
}

#[test]
pub fn test_case_2b() {
    do_test(
        &hex!("c82cf453ffb56d2510aa59815268fbbfa2d06479ee271021384efbc862e2c124"),
        Some("thingamajig"),
        "abc",
    );
}

#[test]
pub fn test_case_3a() {
    do_test(
        &hex!("c75a794e49090b7a9a7144c0acb984e20f4534b4e11e5bbacbe2ec05d44fe85a"),
        None,
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    );
}

#[test]
pub fn test_case_3b() {
    do_test(
        &hex!("facc338851b4ba47ed9d165c358d808fe3189e364b14a095cd8560b85f401d06"),
        Some("thingamajig"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    );
}

#[test]
pub fn test_case_4a() {
    do_test(
        &hex!("43dadfa8368808291ff3bb0b282128305d5ff4606de1f558dbe178390c81adea"),
        None,
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_4b() {
    do_test(
        &hex!("d6fdb861cfb3cd54519fec34371c866351caa664210d151c801c3412b7e11e32"),
        Some("thingamajig"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    );
}

#[test]
pub fn test_case_5a() {
    do_test(
        &hex!("12ccdc15d5eaefa5b9347900b2ac9a9ba7b275deef9d0f372e0701e17e9eb0e2"),
        None,
        from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    );
}

#[test]
pub fn test_case_5b() {
    do_test(
        &hex!("477a83e8a0427c72c3fedb4b9e39a63dcc51b8c8974e0c3c0d4c16db1739be74"),
        Some("thingamajig"),
        from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    );
}
