include ../config.txt

OBJS = arch/amd64/start.o arch/amd64/setjmp.o
RUSTUP_RUN = RUST_TARGET_PATH=../libsyscall/arch rustup run $(CONFIG_RUST_TOOLCHAIN)
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
