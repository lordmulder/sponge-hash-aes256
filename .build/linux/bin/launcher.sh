#!/bin/bash
set -e
readonly INSTALL_DIR="{{SPONGE256SUM_INSTDIR}}"

CPU_FEATURES="undefined"
cpu_features() {
    if [ "${CPU_FEATURES}" = "undefined" ]; then
        CPU_FEATURES="$(lscpu 2>/dev/null | grep -E '^Flags:' | sed -E 's/^Flags:\s*//;s/(\S+)\s*/[\1]/g')"
    fi
    for flag in "$@"; do
        if [[ "${CPU_FEATURES}" != *"[${flag}]"* ]]; then
            return 1
        fi
    done
}

if [ -z "${SPONGE256SUM_ARCH}" ]; then
    case "$(uname -m)" in
    x86_64 | amd64)
        SPONGE256SUM_ARCH="x86_64"
        if cpu_features cx16 lahf_lm popcnt sse4_1 sse4_2 ssse3; then
            SPONGE256SUM_ARCH="x86_64-v2"
            if cpu_features avx avx2 bmi1 bmi2 f16c fma abm movbe xsave; then
                SPONGE256SUM_ARCH="x86_64-v3"
                if cpu_features avx512f avx512bw avx512cd avx512dq avx512vl; then
                    SPONGE256SUM_ARCH="x86_64-v4"
                fi
            fi
        fi
        ;;
    i?86 | x86 | x86pc)
        SPONGE256SUM_ARCH="i586"
        if cpu_features cmov fxsr mmx sse sse2; then
            SPONGE256SUM_ARCH="pentium4"
        fi
        ;;
    aarch64 | arm64)
        SPONGE256SUM_ARCH="aarch64"
        ;;
    riscv64)
        SPONGE256SUM_ARCH="riscv64gc"
        ;;
    *)
        echo "Unknown architecture: $(uname -m)" >&2
        exit 1
        ;;
    esac
fi

exec "${INSTALL_DIR}/sponge256sum-${SPONGE256SUM_ARCH}" "$@"
