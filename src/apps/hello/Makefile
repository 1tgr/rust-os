include ../../config.txt

RUSTUP_RUN = RUST_TARGET_PATH=../../libsyscall/arch rustup run $(CONFIG_RUST_TOOLCHAIN)

all: target/amd64/release/hello

clean:
	$(RUSTUP_RUN) cargo clean

target/amd64/release/hello target/amd64/release/hello.d: Cargo.toml
	$(RUSTUP_RUN) cargo build --release
	sed -i -e "s?`pwd`/??g" target/amd64/release/hello.d

include target/amd64/release/hello.d
