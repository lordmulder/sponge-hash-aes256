#!/bin/sh
set -e

# Prerequisites:
# - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# - rustup target add x86_64-unknown-freebsd i686-unknown-freebsd

if [ "$(uname -s)" != "FreeBSD" ]; then
    echo "Error: This script is supposed to run on a FreeBSD system!"
    exit 1
fi

unset RUSTFLAGS

case "$(uname -m)" in
    i386)
        make MY_OS=freebsd MY_ARCH=i686 MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    amd64)
        make MY_OS=freebsd MY_ARCH="x86_64 i686" MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    *)
        echo "Error: Unknown architecture!"
        exit 1
        ;;
esac
