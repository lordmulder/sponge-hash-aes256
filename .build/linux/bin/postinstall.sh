#!/bin/sh
set -e
DPKG_ARCH="`dpkg --print-architecture`"
case "${DPKG_ARCH}" in
    x86_64|amd64)
        ln -vfs "{{SPONGE256SUM_INSTDIR}}/sponge256sum-x86_64" "/usr/bin/sponge256sum"
        ;;
    i?86|x86|x86pc)
        ln -vfs "{{SPONGE256SUM_INSTDIR}}/sponge256sum-i586" "/usr/bin/sponge256sum"
        ;;
    aarch64|arm64)
        ln -vfs "{{SPONGE256SUM_INSTDIR}}/sponge256sum-aarch64" "/usr/bin/sponge256sum"
        ;;
    *)
        >&2 echo 'Unsupported arch! (${DPKG_ARCH})'
        ;;
esac
