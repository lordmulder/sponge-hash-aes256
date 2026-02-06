#!/bin/sh
set -eu
APP_BASEDIR="$(dirname -- "$(readlink -f -- "$0")")"

if [ "${APPIMAGE_SPONGE256SUM_ARCH:=undefined}" = "undefined" ]; then
    APPIMAGE_SPONGE256SUM_ARCH="aarch64"
fi

export PATH="${APP_BASEDIR}/usr/bin:${PATH}"
exec "${APP_BASEDIR}/usr/bin/sponge256sum-${APPIMAGE_SPONGE256SUM_ARCH}" "$@"
