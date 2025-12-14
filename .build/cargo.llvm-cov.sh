#!/bin/bash
set -e
cd -- "$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")/..")"

for i in lib app; do
	printf -- "-------------------------------\n"
	printf -- "-------------[%3s]-------------\n" "${i}"
	printf -- "-------------------------------\n"
	pushd "${i}"
	cargo clean
	cargo llvm-cov --release --open -- --include-ignored
	popd
done

echo "Completed successfully."
