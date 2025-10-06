SHELL := /bin/sh -ec

.PHONY: all build dist clean

all: clean dist

build:
	rm -rf target && mkdir target && mkdir target/dist
	cd ../../app; \
	for t in x86_64 i686; do \
		cargo build --release --target $$t-unknown-freebsd --verbose; \
		mv -vf target/$$t-unknown-freebsd/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-$$t; \
		chmod 555 $(CURDIR)/target/dist/sponge256sum-$$t; \
	done; \
	cargo pkgid > target/.CARGO_PKGID

docs:
	cd ../../app; \
	cargo doc

dist: build docs
	PKG_VERSION="$$(grep -Eo '[^@]+$$' < ../../app/target/.CARGO_PKGID | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//')"; \
	test -n "$${PKG_VERSION}"; \
	cp -rf ../../app/target/doc target/dist/; \
	cp -f ../../.assets/html/index.html target/dist/doc/index.html; \
	cp -f ../../LICENSE target/dist/LICENSE; \
	git describe --long --always --dirty > target/dist/REVISION; \
	find target/dist -type f -not -name 'sponge256sum-*' -exec chmod 444 {} \;; \
	find target/dist -type d -exec chmod 755 {} \;; \
	( cd target/dist && tar -cJvf ../sponge256sum-$${PKG_VERSION}-freebsd.tar.xz * ); \
	chmod 444 target/sponge256sum-$${PKG_VERSION}-freebsd.tar.xz

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
