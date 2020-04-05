include config.txt

RUSTUP_RUN = RUST_TARGET_PATH=${CURDIR}/libsyscall/arch RUSTFLAGS=-Dwarnings

DIRS_SRC += kernel

DIRS += $(DIRS_SRC)
DIRS += boot

.PHONY: all xargo-build clean test $(DIRS)

all: test

xargo-build:
	$(RUSTUP_RUN) xargo build --target $(CONFIG_TARGET) --release
	(cd libgraphics && $(RUSTUP_RUN) xargo build --target $(CONFIG_TARGET) --release --features test)

clean:
	$(RUSTUP_RUN) xargo clean

$(DIRS): xargo-build
	make -C $@

test: $(DIRS)
	python3 test.py qemu-system-x86_64 -no-kvm -display vnc=:1 -no-reboot -kernel kernel/kernel -initrd boot/initrd.tar
