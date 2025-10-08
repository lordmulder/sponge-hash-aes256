#!/bin/bash
set -e
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

# Prerequisites:
# - apt-get install -y clang lld
# - rustup target add x86_64-unknown-freebsd
# - rustup target add i686-unknown-freebsd
#
# FreeBSD sysroot:
# - https://download.freebsd.org/ftp/releases/i386/14.3-RELEASE/base.txz
# - https://download.freebsd.org/ftp/releases/amd64/14.3-RELEASE/base.txz

if [ "$(uname -s | tr 'A-Z' 'a-z')"  != "linux" ]; then
    echo "Error: This script is supposed to run on a Linux-based system!"
    exit 1
fi

if [ ! -f "/opt/freebsd/sysroot/i386/lib/librt.so.1" ]; then
    echo "Error: FreeBSD 'i386' sysroot not found!"
    exit 1
fi

if [ ! -f "/opt/freebsd/sysroot/amd64/lib/librt.so.1" ]; then
    echo "Error: FreeBSD 'amd64' sysroot not found!"
    exit 1
fi

unset RUSTFLAGS

export CARGO_TARGET_I686_UNKNOWN_FREEBSD_LINKER=clang
export CARGO_TARGET_I686_UNKNOWN_FREEBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-fuse-ld=lld -Clink-arg=--target=i686-unknown-freebsd -Clink-arg=--sysroot=/opt/freebsd/sysroot/i386"
export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER=clang
export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-fuse-ld=lld -Clink-arg=--target=x86_64-unknown-freebsd -Clink-arg=--sysroot=/opt/freebsd/sysroot/amd64"

make MY_OS=freebsd MY_ARCH="x86_64 i686" MY_RUSTFLAGS=
