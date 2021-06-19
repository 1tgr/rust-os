.PHONY: \
	3rdparty-tools-arm32-binary \
	3rdparty-tools-arm32-src \
	3rdparty-tools-amd64-binary \
	3rdparty-tools-amd64-src \
	cargo-install \
	pip-install \
	rustup-toolchain-install \
	setup-arm32 \
	setup-amd64 \
	src-arm32 \
	src-amd64

all: \
	setup-arm32 \
	setup-amd64 \
	src-arm32 \
	src-amd64

setup-arm32: \
	3rdparty-tools-arm32-binary \
	cargo-install \
	pip-install \
	rustup-toolchain-install

setup-amd64: \
	3rdparty-tools-amd64-binary \
	cargo-install \
	pip-install \
	rustup-toolchain-install

3rdparty-tools-arm32-binary:
	$(MAKE) -C 3rdparty tools-arm32-binary
	3rdparty/target/bin/arm-eabi-gcc --version
	3rdparty/target/bin/arm-eabi-ld --version
	3rdparty/target/bin/qemu-system-arm --version

3rdparty-tools-arm32-src:
	$(MAKE) -C 3rdparty tools-arm32-src
	3rdparty/target/bin/arm-eabi-gcc --version
	3rdparty/target/bin/arm-eabi-ld --version
	3rdparty/target/bin/qemu-system-arm --version

3rdparty-tools-amd64-binary:
	$(MAKE) -C 3rdparty tools-amd64-binary
	3rdparty/target/bin/qemu-system-x86_64 --version
	3rdparty/target/bin/x86_64-elf-gcc --version
	3rdparty/target/bin/x86_64-elf-ld --version

3rdparty-tools-amd64-src:
	$(MAKE) -C 3rdparty tools-amd64-src
	3rdparty/target/bin/qemu-system-x86_64 --version
	3rdparty/target/bin/x86_64-elf-gcc --version
	3rdparty/target/bin/x86_64-elf-ld --version

pip-install:
	pip3 install --user -r requirements.txt

cargo-install:
	cargo install --vers 0.3.20 xargo

rustup-toolchain-install:
	rustup toolchain install $$(cat src/rust-toolchain) --component rust-src

src-arm32:
	$(MAKE) -C src test-arm32

src-amd64:
	$(MAKE) -C src test-amd64
