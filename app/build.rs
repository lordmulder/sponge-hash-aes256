// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

// ---------------------------------------------------------------------------
// Build main
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn main() {
    let target_arch = match std::env::var("CARGO_CFG_TARGET_ARCH") {
        Ok(arch) if arch.eq("x86") => "i686".to_string(),
        Ok(arch) => arch,
        Err(_) => panic!("TARGET_ARCH is *not* specified!"),
    };

    let mut winres = winres::WindowsResource::new();
    winres.set_icon(r"resources\app.ico");
    winres.set_manifest_file(&format!(r"resources\app-{target_arch}.manifest"));
    winres.compile().expect("Windows resource compiler has failed!");
}

#[cfg(not(windows))]
fn main() {}
