.PHONY: 3rdparty-tools cargo-install pip-install rustup-toolchain-install setup src

all: setup src

setup: 3rdparty-tools cargo-install pip-install rustup-toolchain-install

3rdparty-tools:
	$(MAKE) -C 3rdparty tools
	env PATH=${PATH}:$(CURDIR)/3rdparty/bin x86_64-elf-ld --version
	env PATH=${PATH}:$(CURDIR)/3rdparty/bin qemu-system-x86_64 --version

pip-install:
	pip3 install --user -r requirements.txt

cargo-install:
	cargo install --vers 0.3.20 xargo

rustup-toolchain-install:
	rustup toolchain install $$(cat src/rust-toolchain) --component rust-src

src:
	env PATH=${PATH}:$(CURDIR)/3rdparty/bin $(MAKE) -C src
