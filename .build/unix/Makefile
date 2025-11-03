MY_ARCH = x86_64
MY_FEATURES = wide
MY_OS = unix
MY_OUTFILENAME = $(MY_OS)
MY_VENDOR = unknown

.PHONY: all build dist doc pkg clean

all: pkg

build: clean
	set -e; \
	rm -rf target && mkdir target && mkdir target/dist; \
	ORIGINAL_DIR="`pwd`"; \
	cd ../../app; \
	for t in $(MY_ARCH); do \
		cargo clean; \
		cargo build --release --features "$(MY_FEATURES)" --target "$$t-$(MY_VENDOR)-$(MY_OS)" --verbose; \
		mv -f target/$$t-$(MY_VENDOR)-$(MY_OS)/release/sponge256sum $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
		chmod 555 $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
	done; \
	for t in $(MY_XARCH); do \
		cargo clean; \
		RUSTC_BOOTSTRAP=1 cargo build --release --features "$(MY_FEATURES)" --target "$$t-$(MY_VENDOR)-$(MY_OS)" -Zbuild-std=std,panic_abort --verbose; \
		mv -f target/$$t-$(MY_VENDOR)-$(MY_OS)/release/sponge256sum $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
		chmod 555 $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
	done

doc: build
	set -e; \
	ORIGINAL_DIR="`pwd`"; \
	cd ../../app; \
	RUSTFLAGS=-Dwarnings cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256; \
	cp -rf target/doc $${ORIGINAL_DIR}/target/dist/; \
	RUSTFLAGS= cargo pkgid | sed 's/^.*@//;s/^[[:space:]]*//;s/[[:space:]]*$$//' > $${ORIGINAL_DIR}/target/.PKG_VERSION; \
	printf '%s\n\n' "`RUSTFLAGS= cargo --version --verbose`" > $${ORIGINAL_DIR}/target/.RUSTC_VERSION; \
	printf '%s\n' "`RUSTFLAGS= cargo rustc -- --version --verbose`" >> $${ORIGINAL_DIR}/target/.RUSTC_VERSION;

dist: doc
	set -e; \
	cp -f ../../.assets/html/index.html target/dist/doc/index.html; \
	cp -f ../../LICENSE target/dist/LICENSE; \
	printf 'Revision: %s\n' "`git describe --long --tags --always --dirty`" > target/dist/BUILD_INFO; \
	printf 'Built: %s\n\n' "`date +'%Y-%m-%d %H:%M:%S'`" >> target/dist/BUILD_INFO; \
	cat target/.RUSTC_VERSION >> target/dist/BUILD_INFO; \
	find target/dist -type f ! -name 'sponge256sum-*' -exec chmod 444 {} \;; \
	find target/dist -type d -exec chmod 755 {} \;

pkg: dist
	set -e; \
	IFS= read -r PKG_VERSION < target/.PKG_VERSION; \
	( cd target/dist && tar -cf ../sponge256sum-$${PKG_VERSION}-$(MY_OUTFILENAME).tar * ); \
	xz -9 target/sponge256sum-$${PKG_VERSION}-$(MY_OUTFILENAME).tar; \
	chmod 444 target/*.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
