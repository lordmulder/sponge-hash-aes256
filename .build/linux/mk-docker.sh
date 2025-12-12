#!/bin/bash
set -eo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

readonly IMAGE_NAME=lordmulder/rust-xbuild
readonly IMAGE_VERS=1.92-trixie-r2

exec docker run --rm -v ../..:/workspace -w /workspace "${IMAGE_NAME}":"${IMAGE_VERS}" make -C .build/linux
