#!/bin/bash
set -e
cd -- "$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")/..")"

cargo clean || goto:error
cargo llvm-cov --features with-mimalloc --workspace --open -- --include-ignored || goto:error

echo "Completed successfully."
