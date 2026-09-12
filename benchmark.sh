#!/bin/bash
set -eo pipefail
cat <<'EOF'
  ____                                 _   _                 _
 / ___| _ __   ___  _ __   __ _  ___  | | | | __ _  ___  ___| |__
 \___ \| '_ \ / _ \| '_ \ / _` |/ _ \ | |_| |/ _` |/ __|/ __| '_ \
  ___) | |_) | (_) | | | | (_| |  __/ |  _  | (_| |\__ \ (__| | | |
 |____/| .__/ \___/|_| |_|\__, |\___| |_| |_|\__,_||___/\___|_| |_|
       |_|                |___/
     _     _____ ____ ____  _____   __
    / \   | ____/ ___|___ \|  ___| / /
   / _ \  |  _| \___ \ __) |___ \ / /_
  / ___ \ | |___ ___) / __/  ___)| '_ \
 /_/   \_\|_____|____/_____|____/ \___/

EOF

# ================================================================
# Constants
# ================================================================

readonly EXE_FILE_NAME="${1:-sponge256sum-x86_64v3}"

readonly DATA_HASH=ec57f40a6ee0b4422180a193612001738f529cc6dfd5f71cfe3c64022599c05c49188083387c277b763bf24e4bbaece184022c4123c42565bcf34a847a7302d4
readonly CHCK_HASH=ebe64861b84cd4d78cc42c7dce45ae8e59bf4df0497bbab5f1870aa4b9f283cc6f6cb991ccee6c8246d9894c8bea6a5dae759219cf17607665b5d567e0911f9d

# ================================================================
# Check Binary
# ================================================================

cd -- "$(dirname -- "$(readlink -f "${BASH_SOURCE[0]}")")"
readonly base_dir="$(pwd)"

case "$(uname -s)" in
    Linux)
        readonly OS_TYPE=linux
        ;;
    CYGWIN*)
        readonly OS_TYPE=windows
        ;;
    *)
        printf "Error: This script is supposed to run on a Linux (or Cygwin) system!\n"
        exit 1
        ;;
esac

for c in bc sha512sum time unxz; do
    if ! command -v "${c}" > /dev/null 2>&1; then
        printf "Error: Required system command \"${c}\" not found. Please install and try again!\n"
        exit 1
    fi
done

if [[ ! -x "${base_dir}/bin/${OS_TYPE}/${EXE_FILE_NAME}" ]]; then
    printf "Error: Executable file \${base_dir}/bin/${OS_TYPE}/${EXE_FILE_NAME}\" not found or access denied!\n"
    exit 1
fi

export LC_ALL=C
export SPONGE256SUM_THREAD_COUNT=$(nproc)

# ================================================================
# Extract
# ================================================================

printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'
printf '\033[96mExtract\033[0m\n'
printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'

if [[ -e "${base_dir}/data/files.tar.xz" ]]; then
    echo "Verifying input file \"files.tar.xz\", please wait..."
else
    echo "Error: File \"${base_dir}/data/files.tar.xz\" not found!"
    exit 1
fi

data_hash="$(sha512sum -b "${base_dir}/data/files.tar.xz" | head -n1 | grep -Po '^[[:xdigit:]]+')"
if [[ "${data_hash}" == "${DATA_HASH}" ]]; then
    echo "File verified successfully."
else
    echo "Error: File hash mismatch detected! (computed hash: ${data_hash})"
    exit 1
fi

readonly work_dir="$(mktemp -d --suffix=.d --tmpdir=/var/tmp)"
trap "cd / && rm -rf \"${work_dir}\"" EXIT
tar -C "${work_dir}" -xJvf "${base_dir}/data/files.tar.xz"

# ================================================================
# Benchmark
# ================================================================

cd -- "${work_dir}"

printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'
printf '\033[96mBenchmark ST\033[0m\n'
printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'

result_ST=9007199254740991

for i in {1..4}; do
    printf "Benchmark run %d of %d in progress, please wait...\n" ${i} 4
    time_file="$(mktemp --suffix=.log)"
    chck_file="$(mktemp --suffix=.out)"
    command time -f '%e' -o "${time_file}" "${base_dir}/bin/${OS_TYPE}/${EXE_FILE_NAME}" -rk . > "${chck_file}"
    chck_digest="$(grep -Po '^[[:xdigit:]]+' "${chck_file}" | sort -f | sha512sum -b | head -n1 | grep -Po '^[[:xdigit:]]+')"
    if [[ "${chck_digest}" != "${CHCK_HASH}" ]]; then
        echo "Error: Hash mismatch detected! (computed hash: ${chck_digest})"
        exit 1
    fi
    time_curr="$(head -n1 "${time_file}")"
    result_ST="$(printf "if (${time_curr} < ${result_ST}) ${time_curr} else ${result_ST}\n" | bc -l)"
    rm -f "${time_file}" "${chck_file}"
done

printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'
printf '\033[96mBenchmark MT\033[0m\n'
printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'

result_MT=9007199254740991

for i in {1..8}; do
    printf "Benchmark run %d of %d in progress, please wait...\n" ${i} 8
    time_file="$(mktemp --suffix=.log)"
    chck_file="$(mktemp --suffix=.out)"
    command time -f '%e' -o "${time_file}" "${base_dir}/bin/${OS_TYPE}/${EXE_FILE_NAME}" -rkm . > "${chck_file}"
    chck_digest="$(grep -Po '^[[:xdigit:]]+' "${chck_file}" | sort -f | sha512sum -b | head -n1 | grep -Po '^[[:xdigit:]]+')"
    if [[ "${chck_digest}" != "${CHCK_HASH}" ]]; then
        echo "Error: Hash mismatch detected! (computed hash: ${chck_digest})"
        exit 1
    fi
    time_curr="$(head -n1 "${time_file}")"
    result_MT="$(printf "if (${time_curr} < ${result_MT}) ${time_curr} else ${result_MT}\n" | bc -l)"
    rm -f "${time_file}" "${chck_file}"
done

# ================================================================
# Results
# ================================================================

printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'
printf '\033[96mResults\033[0m\n'
printf '\033[96m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\033[0m\n'

printf "Result ST: %s\n" "${result_ST}"
printf "Result MT: %s\n" "${result_MT}"
