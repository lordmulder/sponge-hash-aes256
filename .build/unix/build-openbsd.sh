#!/bin/sh
set -e

# Prerequisites:
# - pkg_add rust

if [ "$(uname -s)" != "OpenBSD" ]; then
    echo "Error: This script is supposed to run on a OpenBSD system!"
    exit 1
fi

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

case "$(uname -m)" in
    i386)
        make MY_OS=openbsd MY_ARCH=i686 MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    amd64)
        make MY_OS=openbsd MY_ARCH=x86_64 MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    *)
        echo "Error: Unknown architecture!"
        exit 1
        ;;
esac
