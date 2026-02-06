#!/bin/sh
set -eu
APP_BASEDIR="$(dirname -- "$(readlink -f -- "$0")")"
LSCPU_FLAGS=undefined

cpu_features() {
    if [ "${LSCPU_FLAGS}" = "undefined" ]; then
        LSCPU_FLAGS="$(lscpu 2>/dev/null | grep -E -m1 '^Flags:' | cut -d':' -f2- | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    fi
    for flag in "$@"; do
        if ! printf '%s\n' "${LSCPU_FLAGS}" | grep -qw "${flag}"; then
            return 1
        fi
    done
}

if [ "${APPIMAGE_SPONGE256SUM_ARCH:=undefined}" = "undefined" ]; then
    APPIMAGE_SPONGE256SUM_ARCH="x86_64"
    case "$(uname -m)" in
        x86_64 | amd64)
            if cpu_features cx16 lahf_lm popcnt abm ssse3 sse4_1 sse4_2 f16c fma avx avx2 xsave bmi1 bmi2 movbe; then
                    APPIMAGE_SPONGE256SUM_ARCH="x86_64-v3"
            fi
            if cpu_features aes; then
                APPIMAGE_SPONGE256SUM_ARCH="${APPIMAGE_SPONGE256SUM_ARCH}-aes"
            fi
            ;;
    esac
fi

export PATH="${APP_BASEDIR}/usr/bin:${PATH}"
exec "${APP_BASEDIR}/usr/bin/sponge256sum-${APPIMAGE_SPONGE256SUM_ARCH}" "$@"
