#!/bin/sh
set -e

# Prerequisites:
# - pkgman install rust_bin[_x86]

if [ "$(uname -s)" != "Haiku" ]; then
    echo "Error: This script is supposed to run on a Haiku system!"
    exit 1
fi

export RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
unset RUSTC_BOOTSTRAP

case "$(uname -m)" in
    BePC)
        setarch x86 make MY_OS=haiku MY_ARCH=i686
        ;;
    x86_64)
        make MY_OS=haiku MY_ARCH=x86_64
        ;;
    *)
        echo "Error: Unknown architecture!"
        exit 1
        ;;
esac
