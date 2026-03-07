#!/bin/sh
set -eu
LSCPU_FLAGS=undefined

cpu_features() {
    if [ "${LSCPU_FLAGS}" = "undefined" ]; then
        LSCPU_FLAGS="$(grep -E -m1 '^flags\s*:' < /proc/cpuinfo | cut -d':' -f2- | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    fi
    for flag in "$@"; do
        if ! printf '%s\n' "${LSCPU_FLAGS}" | grep -qw "${flag}"; then
            return 1
        fi
    done
}

if [ "${DOCKER_SPONGE256SUM_ARCH:=:undefined}" = ":undefined" ]; then
    DOCKER_SPONGE256SUM_ARCH="x86_64"
    case "$(uname -m)" in
        x86_64 | amd64)
            if cpu_features cx16 lahf_lm popcnt abm ssse3 sse4_1 sse4_2 f16c fma avx avx2 xsave bmi1 bmi2 movbe; then
                    DOCKER_SPONGE256SUM_ARCH="x86_64-v3"
            fi
            if cpu_features aes; then
                DOCKER_SPONGE256SUM_ARCH="${DOCKER_SPONGE256SUM_ARCH}-aes"
            fi
            ;;
        *)
            echo "[sponge256sum] Warning: Unsupported CPU architecture encountered!" >&2
            ;;
    esac
fi

exec "/usr/libexec/sponge256sum-${DOCKER_SPONGE256SUM_ARCH}" "$@"
