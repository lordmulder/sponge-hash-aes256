#!/bin/sh
set -e

# Prerequisites:
# - pkg_add rust

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

unset RUSTFLAGS
unset RUSTC_BOOTSTRAP

case "$(uname -m)" in
    i86pc)
        case "$(isainfo -n)" in
            i386)
                make MY_OS=$OS_FLAVOR MY_VENDOR=$OS_VENDOR MY_ARCH=i686 MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
                ;;
            amd64)
                make MY_OS=$OS_FLAVOR MY_VENDOR=$OS_VENDOR MY_ARCH=x86_64 MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static"
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
