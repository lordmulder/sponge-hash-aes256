#!/bin/bash
set -eo pipefail
cd -- "$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")")"

readonly IMAGE_NAME=lordmulder/rust-xbuild
readonly IMAGE_VERS=1.92-trixie-r2

exec docker run --rm -v "${PWD}/../..":/workspace:ro -v "${PWD}/out":/workspace/.build/linux/out --tmpfs /tmp/rust-build:rw,exec -w /workspace "${IMAGE_NAME}":"${IMAGE_VERS}" make -C .build/linux
