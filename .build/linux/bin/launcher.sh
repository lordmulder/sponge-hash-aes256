#!/bin/sh
set -e

if [ -z "$SPONGE256SUM_ARCH" ]; then
    case "`uname -m`" in
        x86_64|amd64)   SPONGE256SUM_ARCH="x86_64" ;;
        i?86|x86|x86pc) SPONGE256SUM_ARCH="i586" ;;
        aarch64|arm64)  SPONGE256SUM_ARCH="aarch64" ;;
        *)
            echo "Unknown arch!" >&2
            exit 1 ;;
    esac
fi

exec "{{SPONGE256SUM_INSTDIR}}/sponge256sum-$SPONGE256SUM_ARCH" "$@"
