#!/bin/bash
set -e

for i in lib app; do
	printf -- "-------------------------------\n"
	printf -- "-------------[%3s]-------------\n" "${i}"
	printf -- "-------------------------------\n"
	pushd "../${i}"
	cargo clean
	cargo upgrade
	cargo update
	cargo fmt
	cargo clippy
	cargo build --release
	popd
done

echo "Completed successfully."
