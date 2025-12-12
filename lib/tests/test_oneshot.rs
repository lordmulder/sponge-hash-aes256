// SPDX-License-Identifier: 0BSD
// SpongeHash-AES256
// Copyright (C) 2025 by LoRd_MuldeR <mulder2@gmx.de>

include!("include/utils.rs");

use sponge_hash_aes256::{compute, compute_to_slice, DEFAULT_DIGEST_SIZE};

// ---------------------------------------------------------------------------
// Test functions
// ---------------------------------------------------------------------------

fn do_test(expected: &[u8; DEFAULT_DIGEST_SIZE], info: Option<&str>, message: &str) {
    // compute()
    {
        let digest = compute(info, message.as_bytes());
        assert_digest_eq(&digest, expected);
    }

    // compute_to_slice()
    {
        let mut digest = [0u8; DEFAULT_DIGEST_SIZE];
        compute_to_slice(&mut digest, info, message.as_bytes());
        assert_digest_eq(&digest, expected);
    }
}

// ---------------------------------------------------------------------------
// Test vectors
// ---------------------------------------------------------------------------

include!("include/common.rs");

#[test]
pub fn test_case_6a() {
    static MESSAGE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    static EXPECTED: [&[u8; DEFAULT_DIGEST_SIZE]; 65usize] = [
        &hex!("af46c9b65f45e2a1bd7025e1b108a76ec349aab7485fc6892f83717161dfc40f"),
        &hex!("9a4fa4451c72bf89ecb38dedf7e106ef12c9b76af924586e0dedd269753c1f75"),
        &hex!("169c5df2bc6ad16a7d64d0b5e5aab66ea34a7f14e90274c23070b8e7009c2530"),
        &hex!("ef86f9c553cc9bf42b912533a189abe5ffb4a397f75d012a5d7448fdd179362c"),
        &hex!("65fa305c3e58a6634900ec5f5d45e43d0518f1a61440d31669c729cb5b96c4b1"),
        &hex!("79d456ac51ec4d387c48f53f878cc4c3e301d1461cfd75d8425a0d9f5cecba23"),
        &hex!("077dea8263f44cb632375f80c580fd6f70f6c48090f815d5ad41a7f14cfa04b8"),
        &hex!("8239144de957b51db98602fd96b400a094baaecd59fb18494653386b977b8258"),
        &hex!("e67b757c697f1ad6708e2943032f1c6a71e8cf5879b25c73bf0eaa7c4583b0d4"),
        &hex!("a3879e01d9b9e0f61e03699d291b3e7c5462af1fb84130a92bdb089df52b11ab"),
        &hex!("6ecafe831fb693bcb47487588f7b32954e36acdc30b4878530ead7b459b09a1d"),
        &hex!("20b4ec94f2fe10a154148dd4a5ac06eb23c492974e7c94e8fff741a0cf1c7041"),
        &hex!("11eeb208c0810baead1c983d8a98401ac6b31b957cc31919c34d69de8b636239"),
        &hex!("300cf717d3f27db11809609db8503198d2f467cdf4f4fc082dec426139002d54"),
        &hex!("9d587e1d3deecc5f4c4e096b501d03bf58b168c215f0730c17e8dc87779c5b57"),
        &hex!("e9bdd512ff9f860d3b7ee09b3e61cfe00db30ed0dfa4c254e3f3e5e1b4ef5595"),
        &hex!("05165d2ce552f274dc690563ce3793c0753daefca31c0908d38998dada70a59f"),
        &hex!("c0bcb488051438a524df997185f63234a622a9b90d4ef01cfbc084c2a14be5ca"),
        &hex!("6f0c438bd7c82fbeb19df3d354f22cd70d1179def606ddabb27e113f5981c4ff"),
        &hex!("3a995b7a3f654104e3e48c653a5697b1db9a25f7f96d4dbd9d55ba5f5ce5b40c"),
        &hex!("1b40ae4192f37b56553610687fa121f53e7ec3bf72505ebcc70e8d7b94f47afa"),
        &hex!("3a293228d2bba3618fb4f98b8335a16c186d5cf4711dae1adc3378bfb752d01c"),
        &hex!("51c2897e7751885ad74f45b32424cae1f10529943218da3dcbd9af22e1b67db3"),
        &hex!("f57404b1a0a5528c26c340529900c73ae56dcf977f29357365afefe6f6d86cd1"),
        &hex!("6bc308f79e52e1bdb43590e310e3b09331ba9c33afe190e7fc962ee9cf58a74a"),
        &hex!("c8311afd657e3ce3c89b54a271f5403795be3aa0f6689c5f181f14c94323b6bc"),
        &hex!("6fe3e91034ca7ae477895d9258212a907772ab432982dd7983a03ffd59b29c23"),
        &hex!("1c915296e2f8a4d4797c93b018074e68afbe76f6e3bbbf912769c7f6762f3283"),
        &hex!("28044ccecf6af6ab771cb8cbcea01f9eb70afc5fb88b6453bcbe1520c8882c71"),
        &hex!("801aec9ea8f85e6ea3b59a7903659dcc09b2247b8ce2b43fef7062665b55c1fc"),
        &hex!("483b121d94e5e5f2a3a4ae4b98dd538d2c786b338b7cb1bef6aeca72785a7c9d"),
        &hex!("66ac1023a5451aa0b338ebe9d671e3de10627422b3746427a6ea3552dcff5692"),
        &hex!("78d3e1e19dece2373595b4fcad18c25a45372cc0111f3d5116a2aa4c71d88cd3"),
        &hex!("65b0f956fa6b76ca394a0879891b6ad015f2152185679a7bc8a035eb3d850d70"),
        &hex!("a20bbb7779956853b6d1976028f0bf9f02e702d445c679fdf67bb41dfb835eac"),
        &hex!("f19e0741ca86789de24e2deb38139d532ebe3b66f8398f3d28511901ea41012f"),
        &hex!("df0926cad4f56ea672dc31857f31c90a659941c83ff584d2ccabe075e6cac809"),
        &hex!("8ea259a15fbbfc8b71201d6153b2189b8de7d65a83ca3231ac239a2aab855d01"),
        &hex!("c969a76571aadf4b2422e795ca844d4108e37f179f9b17a0cdf4d1d4982ec876"),
        &hex!("b972f5cd88e4f5f8bc4e1488011de9e66c58360d2c765aa319c0f2f1525886e7"),
        &hex!("bb3ad99076ac997633b00f73a6c20846ad5e319542d3c811af4118fbddd4d3f6"),
        &hex!("78d17d5ed2ce22c5c037bf5fe5b73daa3dea9ff72414cbae0480729ba489ab71"),
        &hex!("99826f5e919de2bfce59de3b7f5060d71ec08abf4997c23e6a5c114e22af8cf1"),
        &hex!("98b68f5fb624b140b33d99cb7bd7bd992b2580fa0ee12da44344147a37168bb7"),
        &hex!("cc6f935af1c5839be7c48ab53b5d63c8aeb0cd4361357a0e24b4e9a2608fb3f0"),
        &hex!("b2a21c8cdb9440b5b95e81e10be4932fa1e7ee26d59d724540f72ca67f92e629"),
        &hex!("0c875c391969ab2365fab23512d7e7cc8d41ab08c6d0b1b05a7b19210a36efb6"),
        &hex!("87b519b40d74e81859f54d7046e5e0a2e1b52b6001d4f30911574328bb91acee"),
        &hex!("d5cafcc73d435fd0719b1f8b198af8ee92d8fbdd9401fe695af94ee839d890be"),
        &hex!("d824605a76a78fe077d46bfdad0714fef271965f3903233b9b5df15297b7fc04"),
        &hex!("9464095449043b1d23a064d89f0a6bd92f445fafdf528fb24aea8fa9773f8376"),
        &hex!("58defd4968e1c0bef361b8824af136d4778796ef9adae885f11bad02c87fc37f"),
        &hex!("1297133d080c6177a4ad17768458a1ad82679486a6ccd0da9377c4fc6523c4c0"),
        &hex!("96c4d67e53c877680f9f4ec31ad60877accde42b5fe051996620535866e867b0"),
        &hex!("5dc731dbde03dfa6ad5ed7be3fa6982e8bdadcb6e505da5c84f12b4071a136e9"),
        &hex!("120058473c57b4f7e70d80eec83c7e651c78bf409aba83491e09894c6c71dfdf"),
        &hex!("c7d78d8992aca9193ce997f944682af50f5adcbef3a09c8e3066ae8f239e9f33"),
        &hex!("e31c20ab2d962d677fea191122e3343c14d3e70abd2703aedebe9400c92c40d9"),
        &hex!("6aa13511d4f9d52be2063a5d6b8501ec51c323d84a00da339374f1e113f8d565"),
        &hex!("1cd1254dceb3575b58112b3921e07e1a8372360c176e069cc1de521f2981d5b9"),
        &hex!("d092b5ca3e959a0643f687a08c36cf984bf2a8172fd0ccf34a6100de08c67f84"),
        &hex!("24907820f72dec9638638bc1d2805484db3a1f4ab14490cfa7c9c4d701b18d60"),
        &hex!("75284853045c236bd90f8e42622b8cc96ead75e308d4ef480f22a2f0952baf42"),
        &hex!("9d98e76cfa14e486eb65ba2bd7683ca925917b951e4277cb6367cd0832e4f4dd"),
        &hex!("5f2c92e78a84fe359f80285307fbf55a399df062c6235bacd7f18810af56f0b6"),
    ];

    for (len, expected) in EXPECTED.iter().enumerate() {
        do_test(expected, None, &MESSAGE[..len]);
    }
}

#[test]
pub fn test_case_6b() {
    static MESSAGE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    static EXPECTED: [&[u8; DEFAULT_DIGEST_SIZE]; 65usize] = [
        &hex!("c26e1a9ada9d9112f5374c5d7e44de04fa3cd6f60e6d1b7b4df875e30004b39b"),
        &hex!("6cef6b1648267578544385a04fb67a4b49bba630bab83983ec083461ef0115b5"),
        &hex!("fb55d161c6a20c671d6755922018971a46ac121fad19a0e09ea1a0f67c68e920"),
        &hex!("28b9d039ac92e12f1b6867085f67708304f8b80d1ccdb0b56347a662cd8f520d"),
        &hex!("ab563cdb74b55e4d05cd13918beb1141873109b701dd44bb34d1c6ffc69b2b34"),
        &hex!("8f5c3192e85b15ed1e1fb6e37a4dcc7f57345de63d2f2b46e1c6a2864a288144"),
        &hex!("df640ffb095aee3e57bdf10540731987738cbb5a07b7a6b2e32151c594d44e42"),
        &hex!("ba7febbef9146b2c598f4cba0d6e0fe2ba981ad7e94d499470725bf779320464"),
        &hex!("4c75885a9f78944de61fac701aaa44fda996723043989c4fb9cbff3b2b40955a"),
        &hex!("87b1c46c5fbe2671e2fb7c493ddfabc75b9d679b935abfea79de53b0cbed95e9"),
        &hex!("1dc41855e58b28547cd7e1f234a1fc3d22f7a2e9aad81ed83ffe620224d6ce47"),
        &hex!("d0d32e67eb910ddc0bcb2f7a881beb26bab0a5fb562e4180421aca36315ca946"),
        &hex!("e918231d94c897ad1314c3beecf8aac88908fca13dfb6d43af6a99fa7713a4fb"),
        &hex!("e574de030f49596830169a0224e39e5a7fa4098204764b2880fab5c7ca957612"),
        &hex!("c42b55512493d91607e7a90da11a666e2ea9c2984702e51c9cf02adb6e4644b2"),
        &hex!("655de3878d2d6e7b401ddb9341722d1f08eb7c388c3f837a0d836b342deade79"),
        &hex!("aaafd9aeb497a903e8ad469d21bdf29881878915cafbf86bcb6ee83de5e17a9b"),
        &hex!("ed26d40706fbfa0e6b8fd53d95f2f3f7b09524da1c0e69151cf74e9b58792f8c"),
        &hex!("0998efc33d19ee3fa58bca65a262f1be6a1ace28a1bcd8c88636d59de7bd4b41"),
        &hex!("bb404b09c19b9f29bc0f9afe7c0b5446f7f0582a8e4efa0837228a70f2d37fe4"),
        &hex!("8a86364b3918e207f44a0de7f52cb3c591119958fe4c3208c6aa67388e8a520b"),
        &hex!("a63db9b264521b569537e47c7b380013cab4189910e58a0e3679ed652b05be20"),
        &hex!("c00f9469e66fbae9b0f4fceb71b4a982e6be155359a471991dc34fd78daa84e9"),
        &hex!("4136c76645698bdef6848e8473762b4d072ab820b1f14fb860d503af7e2dbe3e"),
        &hex!("0407f319aa104a416ffc30d0726827f8f29f34fae9a6cbe8f9e33a6e5fb5a038"),
        &hex!("ef46512111bd6cccfb880ef95b6a28a20f8ab36ca8f171a4a58ae0e8f2434b6e"),
        &hex!("e99a3de89511998753837fc4762c100bd11bd1bca871e5c1371747561ac6968e"),
        &hex!("ecf2f9dae8946b2fecd12c621a8732eb0548318d4706c10bca76291bfa302805"),
        &hex!("c2b6dac6c2af6db1cc9f4013b76a8a0eccc41ca7d622f194a4f94fe9e212181c"),
        &hex!("3fea7ea5b3936e6cb0a549cbafa5e884615fdd0e73cceb474bdea4398553972e"),
        &hex!("1de4dd2af685f25036f571b67c8db62541d6a4b38c451173a6b12458eb2b0cd5"),
        &hex!("5bcbface1d2eda780c074e8467776b7ed515c85f62b589428e459800ccdae198"),
        &hex!("af4fe02589e168915a8dd2dc879fd2e57d9b079315831bc7f4f237188127bf66"),
        &hex!("f1ba082036626808f4fb6ede45b99a85be05ede6851b0615a0a377270a90a78f"),
        &hex!("568cad7d67c955ff93c52be85b8e78ce9ae0ab5f613f017135e3ea3dc87aac02"),
        &hex!("ba6b411d39d7abbd33bb3807b98efce333f78d50ccba73e2013db6d424063f60"),
        &hex!("0c6fee6c9333d41ad679072778fb23056c1e0d284c723973429e6ce4a28b1042"),
        &hex!("ff0c85759dad643a3164c75389f169c99a45d8926d96546ff66d58c48042815d"),
        &hex!("632b823f94fe96ec3fbfc80445b52793efa95dba7b171f11535431379c82df43"),
        &hex!("4abb6651d5f94a1ad6af04a27bcf050118f03afcbe883a800c6681aa80a7c2d9"),
        &hex!("4fcb91261a568eea67d959c7f2092c3af05148f4db1f7e84bd9926fba3499359"),
        &hex!("fa5a194d7b1783f4e412b7f583e343ab83d1f24ff6445b75e2268023b0ec25f8"),
        &hex!("3ae984a0b1305aba362af593235c8a97c1c90ba23517a3fb9487b7b478a1fb9f"),
        &hex!("883cf51fb95d872c03af2fabc88b29a2fce02eeada36efc8d02efbb8f9e83c2c"),
        &hex!("14f3293df33919b044dfb2555c023b5bd5e4391f0ff946097376a81eeff3b6c9"),
        &hex!("81eeacd5d3fc797ff6b0401a0406c26e8c2ee3a4e2c6189f2a308bdd04f4515a"),
        &hex!("8fef447b8fa6acfbc5263e74aab9bbbcc6a44b8feddd96509bc01ba21ded4066"),
        &hex!("28312d6070477035d6cb7cef1f37fb4c2fe0170f2557287bc0c7269fb7a17f1b"),
        &hex!("9c845aee722deecf39b1cabbf1f95987250b0100b83bf46fa0cca9706d0554a3"),
        &hex!("f0450f26f90d08942ef5f442125cdeb77363ebd519a3d8707fbd09480f2c6818"),
        &hex!("7e46b1eba53448f80eefd8f39f4f67d8c639896e77032be39a0c99b015ccdcac"),
        &hex!("52b3f80509237b1b7146fa455b568657c991cedccb5d60d1d7d3b367db744201"),
        &hex!("29167b14359a161192a07f88fb21d250d515282c7fa436dd82cd6f522ecafd0b"),
        &hex!("371e0e0a9eb4cb5b19eedb464c40dd9130c7c34bfb75c82f6a2135782e6b447d"),
        &hex!("d2129768587d133ce30e1dc21307e38973b7c8caf5c297feba2916942a7b7c07"),
        &hex!("68a4d3739a7848cfc3d2ab641c7f3415f1f37cb4fe50478dcab0842ea34c9e19"),
        &hex!("9c9c3ad487ee14f3d647d85b3b5ed1e8a5a3b55f8a090626e3e7b7bf6e7ca672"),
        &hex!("2737e5cb33461068ce26cd092f03837c53f19b89978793364b8eb5d428f51598"),
        &hex!("1df4f081c3d300ba499905dd0b009ae9f7031d5c319c0a67e1326bf7be552f6a"),
        &hex!("f9d673e1daba3c5719875c932f20dac19bf469c897982b21b73fcab9ff9e38ba"),
        &hex!("c9266259021e8f4877c78e8612646dd0610ece9a76a0b730f5b20a5245ebfa96"),
        &hex!("2e73999a6b2799715eb805f59efca1bf7dd6779da8856fc99357c8e18d873ce7"),
        &hex!("a57b9bb7c42e4b2d7a74dcdad77a2c2499d75de95bac098217a6a33797292a05"),
        &hex!("91f6479a119c19c3480a84085dd48c98fe23d7f20a23c9fd7b847fe72f5a29e0"),
        &hex!("2b2a57c7a3011b12f73a3e0b63caedc34b5676ff41baf1b589911717cc81061b"),
    ];

    for (len, expected) in EXPECTED.iter().enumerate() {
        do_test(expected, Some("thingamajig"), &MESSAGE[..len]);
    }
}
