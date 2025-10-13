SHELL = /bin/sh
.SHELLFLAGS = -ec

.ONESHELL:

.PHONY: all build dist pkg clean

all: pkg

build: clean
	rm -rf target && mkdir target && mkdir target/dist
	cd ../../app
	for t in x86_64 i686 aarch64; do
		RUSTFLAGS="-Dwarnings -Clinker=$$t-linux-gnu-gcc -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-O3 -Clink-arg=-Wl,-O3" \
		cargo build --release --target $$t-unknown-linux-musl --verbose
		[ "$$t" = "i686" ] && suffix=pentium4 || suffix=$$t
		mv -vf target/$$t-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-$$suffix
		chmod 555 $(CURDIR)/target/dist/sponge256sum-$$suffix
	done
	for v in v2 v3 v4; do
		RUSTFLAGS="-Dwarnings -Clinker=x86_64-linux-gnu-gcc -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-O3 -Clink-arg=-Wl,-O3 -Ctarget-cpu=x86-64-$$v" \
		cargo build --release --target x86_64-unknown-linux-musl --verbose
		mv -vf target/x86_64-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-x86_64-$$v
		chmod 555 $(CURDIR)/target/dist/sponge256sum-x86_64-$$v
	done
	for p in i586; do
		RUSTFLAGS="-Dwarnings -Clinker=i686-linux-gnu-gcc -Ctarget-feature=+crt-static -Clink-arg=-s -Clink-arg=-O3 -Clink-arg=-Wl,-O3 -Ctarget-cpu=$$p" \
		cargo build --release --target i586-unknown-linux-musl --verbose
		mv -vf target/i586-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-$$p
		chmod 555 $(CURDIR)/target/dist/sponge256sum-$$p
	done
	cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256
	cp -rf target/doc $(CURDIR)/target/dist/
	cargo pkgid | grep -Eo '[^@]+$$' | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//' > $(CURDIR)/target/.PKG_VERSION
	printf '%s\n\n' "`cargo --version --verbose`" > $(CURDIR)/target/.RUSTC_VERSION
	printf '%s\n' "`cargo rustc -- --version --verbose`" >> $(CURDIR)/target/.RUSTC_VERSION

dist: build
	cp -f ../../.assets/html/index.html target/dist/doc/index.html
	cp -f ../../LICENSE target/dist/LICENSE
	printf 'Revision: %s\n' "`git describe --long --tags --always --dirty`" > target/dist/BUILD_INFO
	printf 'Built: %s\n\n' "`date +'%Y-%m-%d %H:%M:%S'`" >> target/dist/BUILD_INFO
	cat target/.RUSTC_VERSION >> target/dist/BUILD_INFO
	find target/dist -type f -not -name 'sponge256sum-*' -exec chmod 444 {} \;
	find target/dist -type d -exec chmod 755 {} \;

pkg: dist
	IFS= read -r PKG_VERSION < target/.PKG_VERSION
	( cd target/dist && XZ_OPT=-9e tar --owner=0 --group=0 -cJvf ../sponge256sum-$${PKG_VERSION}-linux.tar.xz * )
	chmod 444 target/sponge256sum-$${PKG_VERSION}-linux.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
