#!/bin/sh
set -e

# Prerequisites:
# - pkgin install rust

if [ "$(uname -s)" != "NetBSD" ]; then
    echo "Error: This script is supposed to run on a NetBSD system!"
    exit 1
fi

if [ "$(uname -m)" != "amd64" ]; then
    echo "Error: This script is supposed to run on the 'amd64' architecture!"
    exit 1
fi

if [ ! -f "/opt/sysroot/i386/lib/libc.so.12" ]; then
    echo "Error: Sysroot for 'i386' not found!"
    exit 1
fi

unset RUSTFLAGS
export RUSTC_BOOTSTRAP=1

export CARGO_TARGET_X86_64_UNKNOWN_NETBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort"
export CARGO_TARGET_I586_UNKNOWN_NETBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Ctarget-cpu=i586 -Clinker=clang -Clink-arg=--target=i586-unknown-netbsd -Clink-arg=--sysroot=/opt/sysroot/i386"

make MY_OS=netbsd MY_ARCH="i586 x86_64" MY_FEATURES=wide MY_RUSTFLAGS= MY_BUILDOPTS="-Zbuild-std=std,panic_abort --verbose"
