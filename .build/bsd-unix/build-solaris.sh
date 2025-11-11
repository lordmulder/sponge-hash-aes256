#!/bin/sh
set -e

# [Prerequisites (Solaris)]
# - pkg install developer/gcc developer/versioning/git
# - source <(curl -s https://raw.githubusercontent.com/psumbera/solaris-rust/refs/heads/main/sh.rust-web-install)

# [Prerequisites (Illumos)]
# - pkg install developer/build-essential developer/versioning/git
# - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.91.0

if [ "$(uname -s)" != "SunOS" ]; then
    echo "Error: This script is supposed to run on a SunOS system!"
    exit 1
fi

if [ "$(uname -o 2> /dev/null)" == "illumos" ]; then
    OS_FLAVOR=illumos
    OS_VENDOR=unknown
else
    OS_FLAVOR=solaris
    OS_VENDOR=pc
fi

export RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
unset RUSTC_BOOTSTRAP

case "$(uname -m)" in
    i86pc)
        case "$(isainfo -n)" in
            i386)
                make MY_OS=$OS_FLAVOR MY_VENDOR=$OS_VENDOR MY_ARCH=i686
                ;;
            amd64)
                make MY_OS=$OS_FLAVOR MY_VENDOR=$OS_VENDOR MY_ARCH=x86_64
                ;;
            *)
                echo "Error: Unknown architecture!"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Error: Unknown architecture!"
        exit 1
        ;;
esac
