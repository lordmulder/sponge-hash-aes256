#!/bin/sh
set -e

# Prerequisites:
# - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# - rustup target add x86_64-unknown-freebsd i686-unknown-freebsd

if [ "$(uname -s)" != "OpenBSD" ]; then
    echo "Error: This script is supposed to run on a OpenBSD system!"
    exit 1
fi

if [ "$(uname -m)" != "amd64" ]; then
    echo "Error: This script is supposed to run on the 'amd64' architecture!"
    exit 1
fi

if [ ! -f "/opt/sysroot/i386/usr/lib/crt0.o" ]; then
    echo "Error: Sysroot for 'i386' not found!"
   exit 1
fi

if [ ! -f "/opt/sysroot/arm64/usr/lib/crt0.o" ]; then
    echo "Error: Sysroot for 'arm64' not found!"
   exit 1
fi

unset RUSTFLAGS
export RUSTC_BOOTSTRAP=1

export CARGO_TARGET_X86_64_UNKNOWN_OPENBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort"
export CARGO_TARGET_I686_UNKNOWN_OPENBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clinker=../.build/unix/utils/clang-wrapper.sh -Clink-arg=--target=i686-unknown-openbsd -Clink-arg=--sysroot=/opt/sysroot/i386"
export CARGO_TARGET_AARCH64_UNKNOWN_OPENBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clinker=../.build/unix/utils/clang-wrapper.sh -Clink-arg=--target=aarch64-unknown-openbsd -Clink-arg=--sysroot=/opt/sysroot/arm64"

make MY_OS=openbsd MY_ARCH="x86_64 i686 aarch64" MY_FEATURES= MY_RUSTFLAGS= MY_BUILDOPTS="-Zbuild-std=std,panic_abort --verbose"
