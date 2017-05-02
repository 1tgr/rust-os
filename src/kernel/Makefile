OBJS = arch/amd64/start.o arch/amd64/setjmp.o
RUST_TOOLCHAIN = nightly-2017-04-26
TARGET = x86_64-elf

RUSTUP_RUN = RUST_TARGET_PATH=../libsyscall/arch rustup run $(RUST_TOOLCHAIN)
CC = gcc
OBJDUMP = $(TARGET)-objdump
OBJCOPY = $(TARGET)-objcopy
ASFLAGS += -m64
ASFLAGS += -g -I ../newlib/$(TARGET)/include

all: kernel kernel.txt

clean:
	$(RUSTUP_RUN) cargo clean
	rm $(OBJS) kernel kernel.txt

target/amd64-kernel/release/kernel target/amd64-kernel/release/kernel.d: Cargo.toml $(OBJS)
	$(RUSTUP_RUN) cargo build --release
	sed -i -e "s?`pwd`/??g" target/amd64-kernel/release/kernel.d

kernel: target/amd64-kernel/release/kernel
	$(OBJCOPY) -SF elf32-i386 $< $@

kernel.txt: target/amd64-kernel/release/kernel
	$(OBJDUMP) -d $< > $@

.S.o:
	$(CC) $(ASFLAGS) -c -o $@ $<

include target/amd64-kernel/release/kernel.d
