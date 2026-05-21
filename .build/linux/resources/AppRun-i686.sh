#!/bin/sh
set -eu
APP_BASEDIR="$(dirname -- "$(readlink -f -- "$0")")"
CPU_FLAGS=undefined

cpu_features() {
    if [ "${CPU_FLAGS}" = "undefined" ]; then
        CPU_FLAGS="$(cat /proc/cpuinfo 2>/dev/null | grep -Eim1 '^flags[[:space:]]*:' | sed -r 's/^[^:]*:[[:space:]]*//')"
    fi
    for flag in "$@"; do
        if ! printf '%s\n' "${CPU_FLAGS}" | grep -qw "${flag}"; then
            return 1
        fi
    done
}

if [ "${APPIMAGE_SPONGE256SUM_ARCH:=undefined}" = "undefined" ]; then
    APPIMAGE_SPONGE256SUM_ARCH="x86_64"
    case "$(uname -m)" in
        i?86 | x86)
            if cpu_features cmov fxsr mmx sse sse2; then
                APPIMAGE_SPONGE256SUM_ARCH="i686+sse2"
                if cpu_features aes; then
                    APPIMAGE_SPONGE256SUM_ARCH="${APPIMAGE_SPONGE256SUM_ARCH}+aes"
                fi
            fi
            ;;
        x86_64 | amd64)
            APPIMAGE_SPONGE256SUM_ARCH="i686+sse2"
            if cpu_features aes; then
                APPIMAGE_SPONGE256SUM_ARCH="${APPIMAGE_SPONGE256SUM_ARCH}+aes"
            fi
            ;;
        *)
            echo "[sponge256sum] Warning: Unsupported CPU architecture encountered!" >&2
            ;;
    esac
fi

export PATH="${APP_BASEDIR}/usr/bin:${PATH}"
exec "${APP_BASEDIR}/usr/bin/sponge256sum-${APPIMAGE_SPONGE256SUM_ARCH}" "$@"
