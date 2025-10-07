SHELL := /bin/sh -ec
BASE_DIR != pwd

.PHONY: all build docs dist clean

all: clean dist

build:
	rm -rf target && mkdir target && mkdir target/dist
	cd ../../app; \
	for t in x86_64; do \
		cargo build --release --target $$t-unknown-openbsd --verbose; \
		mv -vf target/$$t-unknown-openbsd/release/sponge256sum $(BASE_DIR)/target/dist/sponge256sum-$$t; \
		chmod 555 $(BASE_DIR)/target/dist/sponge256sum-$$t; \
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
	( cd target/dist && tar -cvf ../sponge256sum-$${PKG_VERSION}-openbsd.tar * ); \
	xz -9e target/sponge256sum-$${PKG_VERSION}-openbsd.tar; \
	chmod 444 target/sponge256sum-$${PKG_VERSION}-openbsd.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
