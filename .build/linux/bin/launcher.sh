#!/bin/sh
set -eu
INSTALL_DIR="{{SPONGE256SUM_INSTDIR}}"
LSCPU_FLAGS="$(lscpu 2>/dev/null | grep -E -m1 '^Flags:' | cut -d':' -f2- | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"

cpu_features() {
    for flag in "$@"; do
        if ! printf '%s\n' "${LSCPU_FLAGS}" | grep -qw "${flag}"; then
            return 1
        fi
    done
}

if [ -z "${SPONGE256SUM_ARCH:=}" ]; then
    case "$(uname -m)" in
        x86_64 | amd64)
            SPONGE256SUM_ARCH="x86_64"
            if cpu_features cx16 lahf_lm popcnt abm ssse3 sse4_1 sse4_2 f16c fma avx avx2 xsave bmi1 bmi2 movbe; then
                    SPONGE256SUM_ARCH="x86_64-v3"
            fi
            if cpu_features aes; then
                SPONGE256SUM_ARCH="${SPONGE256SUM_ARCH}-aes"
            fi
            ;;
        i?86 | x86 | x86pc)
            SPONGE256SUM_ARCH="i586"
            if cpu_features cmov fxsr mmx sse sse2; then
                SPONGE256SUM_ARCH="i686-sse2"
                if cpu_features aes; then
                    SPONGE256SUM_ARCH="${SPONGE256SUM_ARCH}-aes"
                fi
            fi
            ;;
        aarch64 | arm64)
            SPONGE256SUM_ARCH="aarch64"
            ;;
        ppc64le)
            SPONGE256SUM_ARCH="powerpc64le"
            ;;
        riscv64)
            SPONGE256SUM_ARCH="riscv64gc"
            ;;
        *)
            echo "Unknown architecture encountered: $(uname -m)" >&2
            exit 1
            ;;
    esac
fi

exec "${INSTALL_DIR}/sponge256sum-${SPONGE256SUM_ARCH}" "$@"
