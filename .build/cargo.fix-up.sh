#!/bin/bash
set -e
cd -- "$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")/..")"

cargo clean
cargo upgrade --recursive --incompatible --pinned
cargo update --workspace
cargo fmt --all
cargo clippy --workspace --all-targets --all-features
cargo build --workspace --release

echo "Completed successfully."
