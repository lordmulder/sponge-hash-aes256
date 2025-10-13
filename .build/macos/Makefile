.PHONY: all dist mkpkg clean

all: mkpkg

dist:
	$(MAKE) -C../unix dist MY_VENDOR=apple MY_OS=darwin MY_ARCH="aarch64 x86_64" MY_RUSTFLAGS="-Dwarnings -Ctarget-feature=+crt-static -Clink-arg=-O3"

mkpkg: dist
	set -e; \
	IFS= read -r PKG_VERSION < ../unix/target/.PKG_VERSION; \
	rm -rf target && mkdir target; \
	hdiutil create -ov -volname "spong256sum $${PKG_VERSION}" -fs HFS+ -srcfolder ../unix/target/dist target/sponge256sum-$${PKG_VERSION}-macos.dmg; \
	hdiutil convert -format UDZO -imagekey zlib-level=9 -o target/~sponge256sum-$${PKG_VERSION}-macos.dmg target/sponge256sum-$${PKG_VERSION}-macos.dmg; \
	mv -vf target/~sponge256sum-$${PKG_VERSION}-macos.dmg target/sponge256sum-$${PKG_VERSION}-macos.dmg; \
	chmod 444 target/sponge256sum-$${PKG_VERSION}-macos.dmg

clean:
	rm -rf target
