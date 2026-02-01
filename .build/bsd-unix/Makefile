MY_ARCH = x86_64
MY_OS = unix
MY_OUTFILENAME = $(MY_OS)
MY_VENDOR = unknown

.PHONY: all build xbuild doc dist pkg clean

all: clean pkg

build:
	set -e; \
	mkdir -p out/target/release; \
	export CARGO_TARGET_DIR="`mktemp -d`"; \
	for t in $(MY_ARCH); do \
		cargo build --release --target "$$t-$(MY_VENDOR)-$(MY_OS)" --verbose; \
		cp -f $${CARGO_TARGET_DIR}/$$t-$(MY_VENDOR)-$(MY_OS)/release/sponge256sum out/target/release/sponge256sum-$$t; \
		chmod 555 out/target/release/sponge256sum-$$t; \
		rm -rf $${CARGO_TARGET_DIR}; \
	done

xbuild:
	set -e; \
	mkdir -p out/target/release; \
	export CARGO_TARGET_DIR="`mktemp -d`"; \
	for t in $(MY_XARCH); do \
		RUSTC_BOOTSTRAP=1 cargo build --release --target "$$t-$(MY_VENDOR)-$(MY_OS)" -Zbuild-std=std,panic_abort --verbose; \
		cp -f $${CARGO_TARGET_DIR}/$$t-$(MY_VENDOR)-$(MY_OS)/release/sponge256sum out/target/release/sponge256sum-$$t; \
		chmod 555 out/target/release/sponge256sum-$$t; \
		rm -rf $${CARGO_TARGET_DIR}; \
	done

doc:
	set -e; \
	mkdir -p out/target/release; \
	export CARGO_TARGET_DIR="`mktemp -d`"; \
	RUSTFLAGS=-Dwarnings cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256; \
	cp -rf $${CARGO_TARGET_DIR}/doc out/target/release/; \
	RUSTFLAGS=-Dwarnings cargo pkgid --package sponge256sum | sed 's/^.*@//;s/^[[:space:]]*//;s/[[:space:]]*$$//' > out/target/.PKG_VERSION; \
	printf '%s\n\n' "`RUSTFLAGS=-Dwarnings cargo --version --verbose`" > out/target/.RUSTC_VERSION; \
	printf '%s\n' "`RUSTFLAGS=-Dwarnings cargo rustc --manifest-path src/Cargo.toml -- --version --verbose`" >> out/target/.RUSTC_VERSION; \
	rm -rf $${CARGO_TARGET_DIR}

dist: build xbuild doc
	set -e; \
	cp -f ../../.assets/html/index.html out/target/release/doc/index.html; \
	cp -f ../../LICENSE out/target/release/LICENSE; \
	printf 'Revision: %s\n' "`git describe --long --tags --always --dirty`" > out/target/release/BUILD_INFO; \
	printf 'Built: %s\n\n' "`date +'%Y-%m-%d %H:%M:%S'`" >> out/target/release/BUILD_INFO; \
	cat out/target/.RUSTC_VERSION >> out/target/release/BUILD_INFO; \
	find out/target/release -type f ! -name 'sponge256sum-*' -exec chmod 444 {} \;; \
	find out/target/release -type d -exec chmod 755 {} \;

pkg: dist
	set -e; \
	IFS= read -r PKG_VERSION < out/target/.PKG_VERSION; \
	( cd out/target/release && tar -cf ../sponge256sum-$${PKG_VERSION}-$(MY_OUTFILENAME).tar * ); \
	xz -9 out/target/sponge256sum-$${PKG_VERSION}-$(MY_OUTFILENAME).tar; \
	chmod 444 out/target/*.tar.xz

clean:
	rm -rf out/target
