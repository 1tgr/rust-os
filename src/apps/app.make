include ../../config.txt

RUSTUP_RUN = RUST_TARGET_PATH=../../libsyscall/arch

.PHONY: all clean

all:
	$(RUSTUP_RUN) xargo build --target $(CONFIG_TARGET) --release

clean:
	$(RUSTUP_RUN) xargo clean
