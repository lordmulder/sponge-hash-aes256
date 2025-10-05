SHELL = /bin/sh
.SHELLFLAGS = -ec

export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER ?= musl-gcc
export CARGO_TARGET_I686_UNKNOWN_LINUX_MUSL_LINKER ?= i686-linux-gnu-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER ?= aarch64-linux-gnu-gcc

.ONESHELL:

.PHONY: all build dist clean

all: clean dist

build:
	rm -rf target && mkdir target && mkdir target/dist
	cd ../../app
	export RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-O3 -Clink-arg=-Wl,-O3"
	for t in x86_64 i686 aarch64; do
		cargo build --release --target $$t-unknown-linux-musl --verbose
		cp -f target/$$t-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-$$t
		chmod 555 $(CURDIR)/target/dist/sponge256sum-$$t
	done
	for v in v2 v3 v4; do
		export RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-O3 -Clink-arg=-Wl,-O3 -Ctarget-cpu=x86_64-$$v"
		cargo build --release --target x86_64-unknown-linux-musl --verbose
		cp -f target/x86_64-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-x86_64-$$v
		chmod 555 $(CURDIR)/target/dist/sponge256sum-x86_64-$$v
	done
	cargo pkgid > target/.CARGO_PKGID

docs:
	cd ../../app
	cargo doc

dist: build docs
	PKG_VERSION="$$(grep -Eo '[^@]+$$' < ../../app/target/.CARGO_PKGID | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//')"
	test -n "$${PKG_VERSION}"
	cp -rf ../../app/target/doc target/dist/
	cp -f ../../.assets/html/index.html target/dist/doc/index.html
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
