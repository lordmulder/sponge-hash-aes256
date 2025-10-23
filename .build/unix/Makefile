MY_OS = unix
MY_ARCH = x86_64
MY_VENDOR = unknown
MY_RUSTFLAGS = -Dwarnings

.PHONY: all build dist pkg clean

all: pkg

build:
	set -e; \
	rm -rf target && mkdir target && mkdir target/dist; \
	ORIGINAL_DIR="`pwd`"; \
	cd ../../app; \
	for t in $(MY_ARCH); do \
		cargo clean; \
		if [ ! -z "$(MY_RUSTFLAGS)" ]; then \
			RUSTFLAGS="$(MY_RUSTFLAGS)" cargo build --release --target "$$t-$(MY_VENDOR)-$(MY_OS)" --verbose; \
		else \
			cargo build --release --target "$$t-$(MY_VENDOR)-$(MY_OS)" --verbose; \
		fi; \
		mv -f target/$$t-$(MY_VENDOR)-$(MY_OS)/release/sponge256sum $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
		chmod 555 $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
	done; \
	cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256; \
	cp -rf target/doc $${ORIGINAL_DIR}/target/dist/; \
	cargo pkgid | sed 's/^.*@//;s/^[[:space:]]*//;s/[[:space:]]*$$//' > $${ORIGINAL_DIR}/target/.PKG_VERSION; \
	printf '%s\n\n' "`cargo --version --verbose`" > $${ORIGINAL_DIR}/target/.RUSTC_VERSION; \
	printf '%s\n' "`cargo rustc -- --version --verbose`" >> $${ORIGINAL_DIR}/target/.RUSTC_VERSION;

dist: build
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
	( cd target/dist && tar -cf ../sponge256sum-$${PKG_VERSION}-$(MY_OS).tar * ); \
	xz -9 target/sponge256sum-$${PKG_VERSION}-$(MY_OS).tar; \
	chmod 444 target/*.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
