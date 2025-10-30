#!/bin/sh
set -e

# Prerequisites:
# - pkg install rust

if [ "$(uname -s)" != "DragonFly" ]; then
    echo "Error: This script is supposed to run on a DragonFly system!"
    exit 1
fi

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

case "$(uname -m)" in
    x86_64)
        make MY_OS=dragonfly MY_ARCH=x86_64 MY_FEATURES= MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
        ;;
    *)
        echo "Error: Unknown architecture!"
        exit 1
        ;;
esac
