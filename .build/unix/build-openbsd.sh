#!/bin/sh
set -e

# [Prerequisites]
# - pkg_add git rust rust-src
# - curl -sSf https://cdn.openbsd.org/pub/OpenBSD/7.8/i386/base78.tgz | tar -C /opt/sysroot/i386 -xzf - ./usr/lib
# - curl -sSf https://cdn.openbsd.org/pub/OpenBSD/7.8/arm64/base78.tgz | tar -C /opt/sysroot/arm64 -xzf - ./usr/lib

if [ "$(uname -s)" != "OpenBSD" ]; then
    echo "Error: This script is supposed to run on a OpenBSD system!"
    exit 1
fi

if [ "$(uname -m)" != "amd64" ]; then
    echo "Error: This script is supposed to run on the 'amd64' architecture!"
    exit 1
fi

if [ ! -f "/opt/sysroot/i386/usr/lib/libc.so.102.0" ]; then
    echo "Error: Sysroot for 'i386' not found!"
   exit 1
fi

if [ ! -f "/opt/sysroot/arm64/usr/lib/libc.so.102.0" ]; then
    echo "Error: Sysroot for 'arm64' not found!"
   exit 1
fi

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

export CARGO_TARGET_X86_64_UNKNOWN_OPENBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort"
export CARGO_TARGET_I686_UNKNOWN_OPENBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clinker=../.build/unix/bin/clang-wrapper.sh -Clink-arg=--target=i686-unknown-openbsd -Clink-arg=--sysroot=/opt/sysroot/i386"
export CARGO_TARGET_AARCH64_UNKNOWN_OPENBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clinker=../.build/unix/bin/clang-wrapper.sh -Clink-arg=--target=aarch64-unknown-openbsd -Clink-arg=--sysroot=/opt/sysroot/arm64"

make MY_OS=openbsd MY_ARCH=x86_64 MY_XARCH="i686 aarch64" MY_FEATURES=
