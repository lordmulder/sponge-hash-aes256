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
	cargo pkgid > target/.CARGO_PKGID

docs:
	cd ../../app
	cargo doc

dist: build docs
	PKG_VERSION="$$(grep -Eo '[^@]+$$' < ../../app/target/.CARGO_PKGID | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//')"
	test -n "$${PKG_VERSION}"
	rm -rf target && mkdir target && mkdir target/dist
	for t in x86_64 i686 aarch64; do
		cp -f ../../app/target/$$t-unknown-linux-musl/release/sponge256sum target/dist/sponge256sum-$$t
		chmod 555 target/dist/sponge256sum-$$t
	done
	cp -rf ../../app/target/doc target/dist/
	cp -f ../.resources/html/index.html target/dist/doc/index.html
	cp -f ../../LICENSE target/dist/LICENSE
	git describe --long --always --dirty > target/dist/REVISION
	find target/dist -type f -not -name 'sponge256sum-*' -exec chmod 444 {} \;
	find target/dist -type d -exec chmod 755 {} \;
	( cd target/dist && XZ_OPT=-9e tar --owner=0 --group=0 -cJvf ../sponge256sum-$${PKG_VERSION}-linux.tar.xz * )
	chmod 444 target/sponge256sum-$${PKG_VERSION}-linux.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
