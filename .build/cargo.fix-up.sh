#!/bin/bash
set -e
cd -- "$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")/..")"

cargo clean
cargo upgrade
cargo update --workspace
cargo fmt --all
cargo clippy --workspace
cargo build --workspace --release

echo "Completed successfully."
