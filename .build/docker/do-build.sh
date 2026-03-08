#!/bin/bash
set -eo pipefail
cd -- "$(realpath -- "$(dirname -- "${BASH_SOURCE[0]}")")"

for name in i686-sse2 x86_64 aarch64; do
	if [ ! -e "../linux/out/target/release/sponge256sum-${name}" ]; then
		echo "Error: Linux binaries not found!"
		exit 1
	fi
done

find bin -mindepth 1 -maxdepth 1 -type d -exec rm -rvf {} \;
rm -rvf bin/LICENSE bin/BUILD_INFO

install -vDm444 -t bin/ ../linux/out/target/release/LICENSE ../linux/out/target/release/BUILD_INFO
install -vDm555 -t bin/386/ ../linux/out/target/release/sponge256sum-i586 ../linux/out/target/release/sponge256sum-i686*
install -vDm555 -t bin/amd64/ ../linux/out/target/release/sponge256sum-x86_64*
install -vDm555 -t bin/arm64/ ../linux/out/target/release/sponge256sum-aarch64*
install -vDm444 -t resources/data/ ../../app/tests/data/text/*.txt ../../app/tests/data/LICENSE

for docker_arch in 386 amd64 arm64; do
	( set -x ; docker buildx build --platform=linux/${docker_arch} . )
done
