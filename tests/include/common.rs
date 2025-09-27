// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#[test]
pub fn test_case_1() {
    do_test(&hex!("1640eed024950214ee2378fa0e0ec14ba84ee84a231d4991634c6f698bd5ee79"), "")
}

#[test]
pub fn test_case_2() {
    do_test(
        &hex!("52fa77818e480fa57a05f4b0eb939204c292b074ab44002624ed02f159bfecf3"),
        "abc",
    )
}

#[test]
pub fn test_case_3() {
    do_test(
        &hex!("6963ccbaf468003edf4ff2aed285e87245f22623687c8d9bdfcca78a2e45389c"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    )
}

#[test]
pub fn test_case_4() {
    do_test(
        &hex!("e5d782b4faee1d7e68ef22de4d676790e67ab99f61b850cfaa89f8926425fa9b"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    )
}

#[test]
pub fn test_case_5() {
    do_test(
        &hex!("095ea698657c59cb1ea408a603f5a74a7b56cad391cef4f818ed306e0a02060c"),
        str::from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    )
}
