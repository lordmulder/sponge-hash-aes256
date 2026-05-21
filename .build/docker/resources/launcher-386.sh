#!/bin/sh
set -eu
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

if [ "${DOCKER_SPONGE256SUM_ARCH:=undefined}" = "undefined" ]; then
    DOCKER_SPONGE256SUM_ARCH="x86_64"
    case "$(uname -m)" in
        i?86 | x86)
            if cpu_features cmov fxsr mmx sse sse2; then
                DOCKER_SPONGE256SUM_ARCH="i686+sse2"
                if cpu_features aes; then
                    DOCKER_SPONGE256SUM_ARCH="${DOCKER_SPONGE256SUM_ARCH}+aes"
                fi
            fi
            ;;
        x86_64 | amd64)
            DOCKER_SPONGE256SUM_ARCH="i686+sse2"
            if cpu_features aes; then
                DOCKER_SPONGE256SUM_ARCH="${DOCKER_SPONGE256SUM_ARCH}+aes"
            fi
            ;;
        *)
            echo "[sponge256sum] Warning: Unsupported CPU architecture encountered!" >&2
            ;;
    esac
fi

exec "/usr/libexec/sponge256sum-${DOCKER_SPONGE256SUM_ARCH}" "$@"
