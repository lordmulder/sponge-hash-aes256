# [Prerequisites]
# - sudo apt install crossbuild-essential-{i386,arm64,riscv64} ruby-dev
# - sudo gem install --no-document fpm
# - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.91.0
# - rustup target add i686-unknown-linux-musl
# - rustup target add aarch64-unknown-linux-musl
# - rustup target add riscv64gc-unknown-linux-musl
# - rustup target add i586-unknown-linux-musl
# - curl -sSf -o /tmp/libllvm21.deb http://ftp.debian.org/debian/pool/main/l/llvm-toolchain-21/libllvm21_21.1.4-5_amd64.deb
# - curl -sSf -o /tmp/llvm-21-linker-tools.deb http://ftp.debian.org/debian/pool/main/l/llvm-toolchain-21/llvm-21-linker-tools_21.1.4-5_amd64.deb
# - curl -sSf -o /tmp/libxml2-16.deb http://ftp.debian.org/debian/pool/main/libx/libxml2/libxml2-16_2.15.1+dfsg-0.3_amd64.deb
# - sudo apt install -y /tmp/libxml2-16.deb /tmp/libllvm21.deb /tmp/llvm-21-linker-tools.deb

SHELL = /bin/sh
.SHELLFLAGS = -ec

ifneq (Linux, $(shell uname -s))
$(error Error: This script is supposed to run on a Linux system!)
endif

ifneq (x86_64, $(shell uname -m))
$(error Error: This script is supposed to run on the 'x86_64' architecture!)
endif

.ONESHELL:
.PHONY: all build dist doc pkg clean

all: pkg

build:
	rm -rf target && mkdir target && mkdir target/dist
	cd ../../app
	unset RUSTC_BOOTSTRAP
	for t in x86_64 i686 aarch64; do
		RUSTFLAGS="-Dwarnings -Clinker=$$t-linux-gnu-gcc -Clinker-plugin-lto=/usr/lib/llvm-21/lib/LLVMgold.so -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-flto=fat -Clink-arg=-Ofast" \
		$(SHELL) -c "cargo clean && cargo build --release --features wide --target $$t-unknown-linux-musl --verbose"
		[ "$$t" = "i686" ] && suffix=pentium4 || suffix=$$t
		mv -vf target/$$t-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-$$suffix
		chmod 555 $(CURDIR)/target/dist/sponge256sum-$$suffix
	done
	for v in v2 v3 v4; do
		RUSTFLAGS="-Dwarnings -Clinker=x86_64-linux-gnu-gcc -Clinker-plugin-lto=/usr/lib/llvm-21/lib/LLVMgold.so -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-flto=fat -Clink-arg=-Ofast -Ctarget-cpu=x86-64-$$v" \
		$(SHELL) -c "cargo clean && cargo build --release --features wide --target x86_64-unknown-linux-musl --verbose"
		mv -vf target/x86_64-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-x86_64-$$v
		chmod 555 $(CURDIR)/target/dist/sponge256sum-x86_64-$$v
	done
	for p in i586; do
		RUSTFLAGS="-Dwarnings -Clinker=i686-linux-gnu-gcc -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-O3 -Ctarget-cpu=$$p" \
		$(SHELL) -c "cargo clean && cargo build --release --features wide --target $$p-unknown-linux-musl --verbose"
		mv -vf target/$$p-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-$$p
		chmod 555 $(CURDIR)/target/dist/sponge256sum-$$p
	done
	for p in riscv64gc; do
		RUSTFLAGS="-Dwarnings -Clinker=riscv64-linux-gnu-gcc -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-O3" \
		$(SHELL) -c "cargo clean && cargo build --release --features wide --target $$p-unknown-linux-musl --verbose"
		mv -vf target/$$p-unknown-linux-musl/release/sponge256sum $(CURDIR)/target/dist/sponge256sum-$$p
		chmod 555 $(CURDIR)/target/dist/sponge256sum-$$p
	done

doc: build
	cd ../../app
	RUSTFLAGS=-Dwarnings cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256
	cp -rf target/doc $(CURDIR)/target/dist/
	RUSTFLAGS= cargo pkgid | grep -Eo '[^@]+$$' | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//' > $(CURDIR)/target/.PKG_VERSION
	printf '%s\n\n' "`RUSTFLAGS= cargo --version --verbose`" > $(CURDIR)/target/.RUSTC_VERSION
	printf '%s\n' "`RUSTFLAGS= cargo rustc -- --version --verbose`" >> $(CURDIR)/target/.RUSTC_VERSION

dist: doc
	cp -f ../../.assets/html/index.html target/dist/doc/index.html
	cp -f ../../LICENSE target/dist/LICENSE
	printf 'Revision: %s\n' "`git describe --long --tags --always --dirty`" > target/dist/BUILD_INFO
	printf 'Built: %s\n\n' "`date +'%Y-%m-%d %H:%M:%S'`" >> target/dist/BUILD_INFO
	cat target/.RUSTC_VERSION >> target/dist/BUILD_INFO
	find target/dist -type f -not -name 'sponge256sum-*' -exec chmod 444 {} \;
	find target/dist -type d -exec chmod 755 {} \;

pkg: dist
	IFS= read -r PKG_VERSION < target/.PKG_VERSION
	cat bin/launcher.sh | sed "s|{{SPONGE256SUM_INSTDIR}}|/opt/sponge256sum/$${PKG_VERSION}|g" | install -D --mode=555 /dev/stdin target/bin/sponge256sum.sh
	( cd target/dist && XZ_OPT=-9e tar --owner=0 --group=0 -cJvf ../sponge256sum-$${PKG_VERSION}-linux.tar.xz * )
	fpm -s dir -t deb -n sponge256sum -v $${PKG_VERSION} -a all -p target/sponge256sum-$${PKG_VERSION}-linux.deb --description "SpongeHash-AES256" --maintainer "MuldeR <mulder2@gmx.de>" --vendor "Muldersoft" --url "https://crates.io/crates/sponge-hash-aes256" --license 0BSD --deb-changelog ../../CHANGELOG.md target/bin/sponge256sum.sh=/usr/bin/sponge256sum target/dist/=/opt/sponge256sum/$${PKG_VERSION}/
	chmod 444 target/sponge256sum-$${PKG_VERSION}-linux.*

clean:
	rm -rf target
	( cd ../../lib && cargo clean )
	( cd ../../app && cargo clean )
