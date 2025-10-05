SHELL := /bin/sh -ec

.PHONY: all build dist clean

all: clean dist

build:
	cd ../../app; \
	for t in x86_64 aarch64; do \
		cargo build --release --target $$t-apple-darwin --verbose; \
	done; \
	cargo pkgid > target/.CARGO_PKGID

docs:
	cd ../../app; \
	cargo doc

dist: build docs
	PKG_VERSION="$$(grep -Eo '[^@]+$$' < ../../app/target/.CARGO_PKGID | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//')"; \
	test -n "$${PKG_VERSION}"; \
	rm -rf target && mkdir target && mkdir target/dist; \
	for t in x86_64 aarch64; do \
		cp -f ../../app/target/$$t-apple-darwin/release/sponge256sum target/dist/sponge256sum-$$t; \
		chmod 555 target/dist/sponge256sum-$$t; \
	done; \
	cp -rf ../../app/target/doc target/dist/; \
	cp -f ../../.assets/html/index.html target/dist/doc/index.html; \
	cp -f ../../LICENSE target/dist/LICENSE; \
	git describe --long --always --dirty > target/dist/REVISION; \
	find target/dist -type f -not -name 'sponge256sum-*' -exec chmod 444 {} \;; \
	find target/dist -type d -exec chmod 755 {} \;; \
	hdiutil create -ov -volname "spong256sum $${PKG_VERSION}" -fs HFS+ -srcfolder target/dist target/sponge256sum-$${PKG_VERSION}-macos.dmg; \
	hdiutil convert -format UDZO -imagekey zlib-level=9 -o target/sponge256sum-$${PKG_VERSION}-macos.compressed.dmg target/sponge256sum-$${PKG_VERSION}-macos.dmg; \
	mv -vf target/sponge256sum-$${PKG_VERSION}-macos.compressed.dmg target/sponge256sum-$${PKG_VERSION}-macos.dmg; \
	chmod 444 target/sponge256sum-$${PKG_VERSION}-macos.dmg

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
