// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#[test]
pub fn test_case_1() {
    do_test(&hex!("cd175e60e0950bfd5916247a0b7efad762d69b0ad41f6898be23952cb3879010"), "")
}

#[test]
pub fn test_case_2() {
    do_test(
        &hex!("4c62a45b7a9ff378f5285cb5896645d00bf7ec2e12c2809e5261c1bdc35a3cca"),
        "abc",
    )
}

#[test]
pub fn test_case_3() {
    do_test(
        &hex!("4b042569f3454bd17971489701232282804835f6ee210656762eab66fd0a3b7a"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    )
}

#[test]
pub fn test_case_4() {
    do_test(
        &hex!("91e98d3331fdc899364dfa2e13342371d72a33915ebadb6c09abf8dfd2355004"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    )
}

#[test]
pub fn test_case_5() {
    do_test(
        &hex!("c8bc507bea55f2aaa7d7913a25c68e8f25a6fb987aadadc4a6bbbeea067d4a6f"),
        str::from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    )
}
