#!/bin/sh
set -e

# Prerequisites:
# - pkgin install rust

if [ "$(uname -s)" != "NetBSD" ]; then
    echo "Error: This script is supposed to run on a NetBSD system!"
    exit 1
fi

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

case "$(uname -m)" in
    i386)
        make MY_OS=netbsd MY_ARCH=i586 MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Ctarget-cpu=i586"
        ;;
    amd64)
        make MY_OS=netbsd MY_ARCH=x86_64 MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    *)
        echo "Error: Unknown architecture!"
        exit 1
        ;;
esac
