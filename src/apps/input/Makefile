include ../../config.txt

RUSTUP_RUN = RUST_TARGET_PATH=../../libsyscall/arch rustup run $(CONFIG_RUST_TOOLCHAIN)

.PHONY: all clean

all:
	$(RUSTUP_RUN) cargo build --release

clean:
	$(RUSTUP_RUN) cargo clean
