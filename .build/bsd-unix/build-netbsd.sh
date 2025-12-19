#!/bin/sh
set -e

# [Prerequisites]
# - /usr/sbin/pkg_add git curl clang
# - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.92.0
# - rustup component add rust-src
# - curl -sSf https://cdn.netbsd.org/pub/NetBSD/NetBSD-10.1/i386/binary/sets/base.tgz | tar -C /opt/sysroot/i386 -xzf - lib usr/lib
# - curl -sSf https://cdn.netbsd.org/pub/NetBSD/NetBSD-10.1/i386/binary/sets/comp.tgz | tar -C /opt/sysroot/i386 -xzf - usr/lib

if [ "$(uname -s)" != "NetBSD" ]; then
    echo "Error: This script is supposed to run on a NetBSD system!"
    exit 1
fi

if [ "$(uname -m)" != "amd64" ]; then
    echo "Error: This script is supposed to run on the 'amd64' architecture!"
    exit 1
fi

if [ ! -f "/opt/sysroot/i386/lib/libc.so.12" ] || [ ! -f "/opt/sysroot/i386/usr/lib/crt0.o" ]; then
    echo "Error: Sysroot for 'i386' not found!"
    exit 1
fi

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

export CARGO_TARGET_X86_64_UNKNOWN_NETBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort"
export CARGO_TARGET_I586_UNKNOWN_NETBSD_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Ctarget-cpu=i586 -Clinker=clang -Clink-arg=--target=i586-unknown-netbsd -Clink-arg=--sysroot=/opt/sysroot/i386"

make MY_OS=netbsd MY_ARCH=x86_64 MY_XARCH=i586
