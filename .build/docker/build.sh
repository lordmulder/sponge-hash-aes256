#!/bin/bash
set -eo pipefail
cd -- "$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")")"

if [ ! -e "../linux/out/target/release/sponge256sum-x86_64" ]; then
	echo "Error: Linux binaries not found!"
	exit 1
fi

rm -rf bin/386 bin/amd64 bin/arm64 bin/LICENSE bin/BUILD_INFO
mkdir -p bin/386 bin/amd64 bin/arm64

cp -vf ../linux/out/target/release/LICENSE ../linux/out/target/release/BUILD_INFO bin/
cp -vf ../linux/out/target/release/sponge256sum-i586 ../linux/out/target/release/sponge256sum-i686* bin/386/
cp -vf ../linux/out/target/release/sponge256sum-x86_64* bin/amd64/
cp -vf ../linux/out/target/release/sponge256sum-aarch64* bin/arm64/
cp -vf ../../app/tests/data/text/*.txt resources/data/

for docker_arch in 386 amd64 arm64; do
	( set -x ; docker buildx build --platform=linux/${docker_arch} . )
done
