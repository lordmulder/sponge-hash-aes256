SHELL = /bin/sh
SHELLFLAGS = -e

.PHONY: all build docs dist clean

all: clean dist

build:
	rm -rf target && mkdir target && mkdir target/dist; \
    ORIGINAL_DIR="$$(pwd)"; \
	cd ../../app; \
	for t in x86_64; do \
		cargo build --release --target $$t-unknown-illumos --verbose; \
		mv -f target/$$t-unknown-illumos/release/sponge256sum $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
		chmod 555 $${ORIGINAL_DIR}/target/dist/sponge256sum-$$t; \
	done; \
	cargo pkgid > target/.CARGO_PKGID; \
	printf '%s\n\n%s\n' "$$(cargo --version --verbose)" "$$(cargo rustc -- --version --verbose)" > target/.RUSTC_VERSION

docs:
	cd ../../app; \
	cargo doc

dist: build docs
	PKG_VERSION="$$(grep -Eo '[^@]+$$' < ../../app/target/.CARGO_PKGID | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//')"; \
	test -n "$${PKG_VERSION}"; \
	cp -rf ../../app/target/doc target/dist/; \
	cp -f ../../.assets/html/index.html target/dist/doc/index.html; \
	cp -f ../../LICENSE target/dist/LICENSE; \
	printf 'Revision: %s\n' "$$(git describe --long --always --dirty)" > target/dist/BUILD_INFO; \
	printf 'Built on %s\n\n' "$$(date +'%Y-%m-%d %H:%M:%S')" >> target/dist/BUILD_INFO; \
	cat ../../app/target/.RUSTC_VERSION >> target/dist/BUILD_INFO; \
	find target/dist -type f -not -name 'sponge256sum-*' -exec chmod 444 {} \;; \
	find target/dist -type d -exec chmod 755 {} \;; \
	( cd target/dist && tar -cJvf ../sponge256sum-$${PKG_VERSION}-illumos.tar.xz * ); \
	chmod 444 target/sponge256sum-$${PKG_VERSION}-illumos.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
