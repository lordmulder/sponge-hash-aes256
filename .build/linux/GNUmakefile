SHELL = /bin/sh
.SHELLFLAGS = -ec

.ONESHELL:

.PHONY: all build dist clean

all: clean dist

build:
	cd ../../app
	for t in x86_64 i686 aarch64; do
		cargo build --release --target $$t-unknown-linux-musl
	done
	cargo metadata --format-version 1 > target/.metadata.json

docs:
	cd ../../app
	cargo doc

dist: build docs
	PKG_VERSION="$$(jq -r '.packages[] | select(.name == "sponge256sum") | .version' < ../../app/target/.metadata.json)"
	test -n "$${PKG_VERSION}"
	rm -rf target && mkdir target && mkdir target/dist
	for t in x86_64 i686 aarch64; do
		install -s --mode=555 --strip-program=$$t-linux-gnu-strip ../../app/target/$$t-unknown-linux-musl/release/sponge256sum target/dist/sponge256sum-$$t
	done
	cp -rf ../../app/target/doc target/dist/ &&	find target/dist/doc -type f -exec chmod 444 {} \;
	install --mode=444 ../.resources/html/index.html target/dist/doc/index.html
	install --mode=444 ../../LICENSE target/dist/LICENSE
	( cd target/dist && XZ_OPT=-9e tar --owner=0 --group=0 -cJvf ../sponge256sum-$${PKG_VERSION}-linux.tar.xz * )
	chmod 444 target/sponge256sum-$${PKG_VERSION}-linux.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
