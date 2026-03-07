#!/bin/sh
set -eu
LSCPU_FLAGS=undefined

if [ "${DOCKER_SPONGE256SUM_ARCH:=:undefined}" = ":undefined" ]; then
    DOCKER_SPONGE256SUM_ARCH="aarch64"
    case "$(uname -m)" in
        aarch64 | arm64)
            ;;
        *)
            echo "[sponge256sum] Warning: Unsupported CPU architecture encountered!" >&2
            ;;
    esac
fi

exec "/usr/libexec/sponge256sum-${DOCKER_SPONGE256SUM_ARCH}" "$@"
