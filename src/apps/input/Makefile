include ../../config.txt

RUSTUP_RUN = RUST_TARGET_PATH=../../libsyscall/arch rustup run $(CONFIG_RUST_TOOLCHAIN)

all: target/amd64/release/input

clean:
	$(RUSTUP_RUN) cargo clean

target/amd64/release/input target/amd64/release/input.d: Cargo.toml
	$(RUSTUP_RUN) cargo build --release
	sed -i -e "s?`pwd`/??g" target/amd64/release/input.d

include target/amd64/release/input.d
