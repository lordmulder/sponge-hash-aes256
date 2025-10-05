#!/bin/bash
set -e
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

if [ ! -f "/opt/freebsd/sysroot/i386/lib/librt.so.1" ]; then
    echo "Error: FreeBSD 'i386' sysroot not found!"
    exit 1
fi

if [ ! -f "/opt/freebsd/sysroot/amd64/lib/librt.so.1" ]; then
    echo "Error: FreeBSD 'amd64' sysroot not found!"
    exit 1
fi

export CARGO_TARGET_I686_UNKNOWN_FREEBSD_LINKER=clang
export CARGO_TARGET_I686_UNKNOWN_FREEBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-fuse-ld=lld -Clink-arg=--target=i686-unknown-freebsd -Clink-arg=--sysroot=/opt/freebsd/sysroot/i386"
export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER=clang
export CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-fuse-ld=lld -Clink-arg=--target=x86_64-unknown-freebsd -Clink-arg=--sysroot=/opt/freebsd/sysroot/amd64"

make
