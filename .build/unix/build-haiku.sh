#!/bin/sh
set -e

# Prerequisites:
# - pkgman install rust_bin[_x86]

if [ "$(uname -s)" != "Haiku" ]; then
    echo "Error: This script is supposed to run on a Haiku system!"
    exit 1
fi

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

case "$(uname -m)" in
    BePC)
        setarch x86 make MY_OS=haiku MY_ARCH=i686 MY_FEATURES=wide MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    x86_64)
        make MY_OS=haiku MY_ARCH=x86_64 MY_FEATURES=wide MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    *)
        echo "Error: Unknown architecture!"
        exit 1
        ;;
esac
