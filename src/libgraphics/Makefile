include ../config.txt

RUSTUP_RUN = RUST_TARGET_PATH=${CURDIR}/../libsyscall/arch RUSTFLAGS=-Dwarnings

.PHONY: all clean

all:
	$(RUSTUP_RUN) xargo build --target $(CONFIG_TARGET) --release --features test

clean:
	$(RUSTUP_RUN) xargo clean
