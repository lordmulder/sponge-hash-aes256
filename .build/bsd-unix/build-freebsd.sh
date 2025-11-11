#!/bin/sh
set -e

# [Prerequisites]
# - pkg install -y git curl
# - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.91.0
# - rustup target add x86_64-unknown-freebsd i686-unknown-freebsd
# - rustup component add rust-src
# - curl -sSf https://download.freebsd.org/ftp/releases/arm64/14.3-RELEASE/base.txz | tar -C /opt/sysroot/arm64 -xJ lib usr/lib

if [ "$(uname -s)" != "FreeBSD" ]; then
    echo "Error: This script is supposed to run on a FreeBSD system!"
    exit 1
fi

if [ "$(uname -m)" != "amd64" ]; then
    echo "Error: This script is supposed to run on the 'amd64' architecture!"
    exit 1
fi

if [ ! -f "/opt/sysroot/arm64/lib/librt.so.1" ]; then
    echo "Error: Sysroot for 'arm64' not found!"
    exit 1
fi

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort"
export CARGO_TARGET_I686_UNKNOWN_FREEBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort"
export CARGO_TARGET_AARCH64_UNKNOWN_FREEBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=--sysroot=/opt/sysroot/arm64"

make MY_OS=freebsd MY_ARCH="x86_64 i686" MY_XARCH=aarch64
