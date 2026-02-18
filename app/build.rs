// SPDX-License-Identifier: 0BSD
// sponge256sum
// Copyright (C) 2025-2026 by LoRd_MuldeR <mulder2@gmx.de>

// ---------------------------------------------------------------------------
// Build main
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn main() {
    let mut winres = winres::WindowsResource::new();
    winres.set_icon(r"resources\app.ico");
    winres.set_manifest_file(r"resources\app.manifest");
    winres.compile().expect("Winres failed!");
}

#[cfg(not(windows))]
fn main() {}
