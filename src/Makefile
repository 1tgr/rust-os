RUSTUP_RUN = RUST_TARGET_PATH=${CURDIR}/syscall/arch RUSTFLAGS=-Dwarnings

.PHONY: all xargo-build-arm32 xargo-build-amd64 clean test-arm32 test-amd64 $(DIRS)

all: test-arm32 test-amd64

xargo-build-arm32:
	$(RUSTUP_RUN) xargo build --target arm32 --release --bins

xargo-build-amd64:
	$(RUSTUP_RUN) xargo build --target amd64 --release --bins

clean:
	$(RUSTUP_RUN) xargo clean

kernel-arm32:
	$(MAKE) -C kernel/arch/arm32

kernel-amd64:
	$(MAKE) -C kernel/arch/amd64

boot-arm32: kernel-arm32 xargo-build-arm32
	$(MAKE) -C boot/arm32

boot-amd64: kernel-amd64 xargo-build-amd64
	$(MAKE) -C boot/amd64

test-arm32: kernel-arm32
	python3 test.py qemu-system-arm -M raspi2 -kernel kernel/target/arm32/release/kernel

test-amd64: boot-amd64
	python3 test.py --screenshot qemu-system-x86_64 -no-kvm -display vnc=:1 -no-reboot -kernel kernel/target/amd64-kernel/stripped/kernel -initrd boot/amd64/initrd.tar
