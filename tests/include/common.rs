// SPDX-License-Identifier: 0BSD
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

#[test]
pub fn test_case_1() {
    do_test(&hex!("d1c9cc837cdff9096cbb96ef1aa539508fc152e49e7ce32f754e5298cbac2c40"), "")
}

#[test]
pub fn test_case_2() {
    do_test(
        &hex!("22eeabfbd328997670b7d791d806c93113d9727ba1949bb1dff837b3dcea44f7"),
        "abc",
    )
}

#[test]
pub fn test_case_3() {
    do_test(
        &hex!("f8a7116886fd9d4cdf23cbbb183ff334304b24bc1c91f2525ae09b9befd3306e"),
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
    )
}

#[test]
pub fn test_case_4() {
    do_test(
        &hex!("3a60559daa1ccaea56211f0f691044b09c2bf388cb2a99db0100f97aced564d6"),
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
    )
}

#[test]
pub fn test_case_5() {
    do_test(
        &hex!("66088eea4ce8377017d0170a6e219fcaa7282c57152f06cdcacfa6466c3778ff"),
        str::from_utf8(&[0x61u8; 1000000usize]).unwrap(),
    )
}
