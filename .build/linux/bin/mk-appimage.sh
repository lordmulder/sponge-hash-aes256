#!/bin/bash
set -eu

if [[ $# -lt 4 || -z "${1}" || -z "${2}" || -z "${3}" || -z "${4}" ]]; then
    echo "Usage: ${0} VERSION ARCH OUTPUT_FILE INPUT_FILES..."
    exit 1
fi

readonly SCRIPT_PATH="$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")"
readonly MY_APP_VERS="${1}"
readonly MY_APP_ARCH="${2}"
readonly OUTPUT_FILE="${3}"

shift 3

if [ -x "${SCRIPT_PATH}/appimagetool.AppImage" ]; then
    readonly APPIMAGETOOL_BIN="${SCRIPT_PATH}/appimagetool.AppImage"
elif [ -x "/opt/appimage/appimagetool.AppImage" ]; then
    readonly APPIMAGETOOL_BIN="/opt/appimage/appimagetool.AppImage"
else
    echo "Error: Required program 'appimagetool.AppImage' not found!"
    exit 1
fi

if [ -f "${SCRIPT_PATH}/runtimes/runtime-${MY_APP_ARCH}" ]; then
    readonly RUNTIME_FILE="${SCRIPT_PATH}/runtimes/runtime-${MY_APP_ARCH}"
elif [ -f "/opt/appimage/runtimes/runtime-${MY_APP_ARCH}" ]; then
    readonly RUNTIME_FILE="/opt/appimage/runtimes/runtime-${MY_APP_ARCH}"
else
    readonly RUNTIME_FILE=":undefined"
fi

if [ ! -f "${SCRIPT_PATH}/resources/AppRun-${MY_APP_ARCH}.sh" ]; then
    echo "Error: Launcher script for architecture \"${MY_APP_ARCH}\" is not available!"
    exit 1
fi

readonly BUILD_DIR="$(mktemp -d)"
if [[ -z "${BUILD_DIR}" || ! -d "${BUILD_DIR}" ]]; then
    echo "Error: Failed to create temp directory!"
    exit 1
fi

trap "rm -rf \"${BUILD_DIR}\"" EXIT
readonly MY_APP_DIR="${BUILD_DIR}/sponge256sum.AppDir"
mkdir -p "${MY_APP_DIR}/usr/bin"

install -v --mode 555 "${SCRIPT_PATH}/resources/AppRun-${MY_APP_ARCH}.sh" "${MY_APP_DIR}/AppRun"
install -v --mode 444 "${SCRIPT_PATH}/resources/sponge256sum.desktop" "${MY_APP_DIR}/sponge256sum.desktop"
install -v --mode 444 "${SCRIPT_PATH}/resources/sponge256sum.png" "${MY_APP_DIR}/sponge256sum.png"
install -v --mode 444 "${SCRIPT_PATH}/../../../LICENSE" "${MY_APP_DIR}/LICENSE"
install -v --mode 444 "${SCRIPT_PATH}/../../../README.md" "${MY_APP_DIR}/README.md"

while [ $# -gt 0 ]; do
    install -v --mode 555 "${1}" "${MY_APP_DIR}/usr/bin/$(basename -- "${1}")"
    shift
done

if [ "${RUNTIME_FILE}" != ":undefined" ]; then
    ( set -x; ARCH="${MY_APP_ARCH}" VERSION="${MY_APP_VERS}" "${APPIMAGETOOL_BIN}" --no-appstream --runtime-file "${RUNTIME_FILE}" "${MY_APP_DIR}" "${OUTPUT_FILE}" )
else
    ( set -x; ARCH="${MY_APP_ARCH}" VERSION="${MY_APP_VERS}" "${APPIMAGETOOL_BIN}" --no-appstream "${MY_APP_DIR}" "${OUTPUT_FILE}" )
fi
