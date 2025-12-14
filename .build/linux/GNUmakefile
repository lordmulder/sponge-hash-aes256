# [Prerequisites]
# Please see: https://github.com/lordmulder/rust-xbuild/blob/master/Dockerfile

SHELL = /bin/bash
.SHELLFLAGS = -eo pipefail -c

ifneq (Linux, $(shell uname -s))
$(error Error: This script is supposed to run on a Linux system!)
endif

ifneq (x86_64, $(shell uname -m))
$(error Error: This script is supposed to run on the 'x86_64' architecture!)
endif

unexport RUSTFLAGS
unexport RUSTC_BOOTSTRAP

.ONESHELL:
.PHONY: all build vbuild wbuild xbuild dist doc pkg clean

all: clean pkg

build:
	mkdir -p out/target/release /tmp/rust-build
	cd ../../app
	for t in x86_64 i686 aarch64; do
		export CARGO_TARGET_DIR="$$(mktemp --tmpdir=/tmp/rust-build -d)"
		RUSTFLAGS="-Dwarnings -Clinker=$$t-linux-gnu-gcc -Clinker-plugin-lto=/usr/lib/llvm-21/lib/LLVMgold.so -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-flto=fat -Clink-arg=-Ofast" \
		cargo build --release --target $$t-unknown-linux-musl --verbose
		[ "$$t" = "i686" ] && suffix=i686-sse2 || suffix=$$t
		install -v --mode 555 $${CARGO_TARGET_DIR}/$$t-unknown-linux-musl/release/sponge256sum $(CURDIR)/out/target/release/sponge256sum-$$suffix
		rm -rf $${CARGO_TARGET_DIR}
	done

vbuild:
	mkdir -p out/target/release /tmp/rust-build
	cd ../../app
	for v in v2 v3; do
		export CARGO_TARGET_DIR="$$(mktemp --tmpdir=/tmp/rust-build -d)"
		RUSTFLAGS="-Dwarnings -Clinker=x86_64-linux-gnu-gcc -Clinker-plugin-lto=/usr/lib/llvm-21/lib/LLVMgold.so -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-flto=fat -Clink-arg=-Ofast -Ctarget-cpu=x86-64-$$v" \
		cargo build --release --target x86_64-unknown-linux-musl --verbose
		install -v --mode 555 $${CARGO_TARGET_DIR}/x86_64-unknown-linux-musl/release/sponge256sum $(CURDIR)/out/target/release/sponge256sum-x86_64-$$v
		rm -rf $${CARGO_TARGET_DIR}
	done

wbuild:
	mkdir -p out/target/release /tmp/rust-build
	cd ../../app
	for t in x86_64 i686; do
		export CARGO_TARGET_DIR="$$(mktemp --tmpdir=/tmp/rust-build -d)"
		RUSTFLAGS="-Dwarnings -Clinker=$$t-linux-gnu-gcc -Clinker-plugin-lto=/usr/lib/llvm-21/lib/LLVMgold.so -Ctarget-feature=+crt-static,+aes -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-flto=fat -Clink-arg=-Ofast" \
		cargo build --release --target $$t-unknown-linux-musl --verbose
		[ "$$t" = "i686" ] && arch_suffix=i686-sse2-aes_ni || arch_suffix=$$t-aes_ni
		install -v --mode 555 $${CARGO_TARGET_DIR}/$$t-unknown-linux-musl/release/sponge256sum $(CURDIR)/out/target/release/sponge256sum-$$arch_suffix
		rm -rf $${CARGO_TARGET_DIR}
	done
	for v in v2 v3; do
		export CARGO_TARGET_DIR="$$(mktemp --tmpdir=/tmp/rust-build -d)"
		RUSTFLAGS="-Dwarnings -Clinker=x86_64-linux-gnu-gcc -Clinker-plugin-lto=/usr/lib/llvm-21/lib/LLVMgold.so -Ctarget-feature=+crt-static,+aes -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-flto=fat -Clink-arg=-Ofast -Ctarget-cpu=x86-64-$$v" \
		cargo build --release --target x86_64-unknown-linux-musl --verbose
		install -v --mode 555 $${CARGO_TARGET_DIR}/x86_64-unknown-linux-musl/release/sponge256sum $(CURDIR)/out/target/release/sponge256sum-x86_64-$$v-aes_ni
		rm -rf $${CARGO_TARGET_DIR}
	done

xbuild:
	mkdir -p out/target/release /tmp/rust-build
	cd ../../app
	for p in i586:i686 riscv64gc:riscv64 powerpc64le:powerpc64le; do
		IFS=":" read -r -a q <<< "$$p"
		export CARGO_TARGET_DIR="$$(mktemp --tmpdir=/tmp/rust-build -d)"
		RUSTFLAGS="-Dwarnings -Clinker=$${q[1]}-linux-gnu-gcc -Ctarget-feature=+crt-static -Copt-level=3 -Cdebuginfo=none -Ccodegen-units=1 -Clto=fat -Cpanic=abort -Clink-arg=-s -Clink-arg=-flto=fat -Clink-arg=-Ofast" \
		cargo build --release --target $${q[0]}-unknown-linux-musl --verbose
		install -v --mode 555 $${CARGO_TARGET_DIR}/$${q[0]}-unknown-linux-musl/release/sponge256sum $(CURDIR)/out/target/release/sponge256sum-$${q[0]}
		rm -rf $${CARGO_TARGET_DIR}
	done

doc:
	mkdir -p out/target/release /tmp/rust-build
	cd ../../app
	export CARGO_TARGET_DIR="$$(mktemp --tmpdir=/tmp/rust-build -d)"
	RUSTFLAGS=-Dwarnings cargo doc --no-deps --package sponge256sum --package sponge-hash-aes256
	cp -rf $${CARGO_TARGET_DIR}/doc $(CURDIR)/out/target/release/
	RUSTFLAGS= cargo pkgid | grep -Eo '[^@]+$$' | sed 's/^[[:space:]]*//;s/[[:space:]]*$$//' > $(CURDIR)/out/target/.PKG_VERSION
	printf '%s\n\n' "$$(RUSTFLAGS= cargo --version --verbose)" > $(CURDIR)/out/target/.RUSTC_VERSION
	printf '%s\n' "$$(RUSTFLAGS= cargo rustc -- --version --verbose)" >> $(CURDIR)/out/target/.RUSTC_VERSION
	rm -rf $${CARGO_TARGET_DIR}

dist: build vbuild wbuild xbuild doc
	cp -f ../../.assets/html/index.html out/target/release/doc/index.html
	cp -f ../../LICENSE out/target/release/LICENSE
	printf 'Revision: %s\n' "$$(git describe --long --tags --always --dirty)" > out/target/release/BUILD_INFO
	printf 'Built: %s\n\n' "$$(date +'%Y-%m-%d %H:%M:%S')" >> out/target/release/BUILD_INFO
	cat out/target/.RUSTC_VERSION >> out/target/release/BUILD_INFO
	find out/target/release -type f -not -name 'sponge256sum-*' -exec chmod 444 {} \;
	find out/target/release -type d -exec chmod 755 {} \;

pkg: dist
	IFS= read -r PKG_VERSION < out/target/.PKG_VERSION
	cat bin/launcher.sh | sed "s|{{SPONGE256SUM_INSTDIR}}|/opt/sponge256sum/$${PKG_VERSION}|g" | install -D --mode=555 /dev/stdin out/target/bin/sponge256sum.sh
	( cd out/target/release && XZ_OPT=-9e tar --owner=0 --group=0 -cJvf ../sponge256sum-$${PKG_VERSION}-linux.tar.xz * )
	fpm -s dir -t deb -n sponge256sum -v $${PKG_VERSION} -a all -p out/target/sponge256sum-$${PKG_VERSION}-linux.deb --description "SpongeHash-AES256" --maintainer "MuldeR <mulder2@gmx.de>" --vendor "Muldersoft" --url "https://crates.io/crates/sponge-hash-aes256" --license 0BSD --deb-changelog ../../CHANGELOG.md out/target/bin/sponge256sum.sh=/usr/bin/sponge256sum out/target/release/=/opt/sponge256sum/$${PKG_VERSION}/
	chmod 444 out/target/sponge256sum-$${PKG_VERSION}-linux.*

clean:
	rm -rf out/target
