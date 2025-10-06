#!/bin/bash
set -e
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

if [[ ! -f "/opt/netbsd/sysroot/amd64/usr/lib/librt.so.1" || ! -f "/opt/netbsd/sysroot/amd64/usr/lib/crt0.o" ]]; then
    echo "Error: NetBSD 'amd64' sysroot not found!"
    exit 1
fi

unset RUSTFLAGS

export CARGO_TARGET_X86_64_UNKNOWN_NETBSD_LINKER=clang
export CARGO_TARGET_X86_64_UNKNOWN_NETBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-fuse-ld=lld -Clink-arg=--target=x86_64-unknown-netbsd -Clink-arg=--sysroot=/opt/netbsd/sysroot/amd64"

make
