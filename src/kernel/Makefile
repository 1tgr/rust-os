include ../config.txt

OBJS = arch/amd64/start.o arch/amd64/setjmp.o
RUSTUP_RUN = RUST_TARGET_PATH=../libsyscall/arch rustup run $(CONFIG_RUST_TOOLCHAIN)
CC = gcc
OBJDUMP = $(TARGET)-objdump
OBJCOPY = $(TARGET)-objcopy
ASFLAGS += -m64
ASFLAGS += -g -I ../newlib/$(TARGET)/include

.PHONY: all clean

all: $(OBJS)
	$(RUSTUP_RUN) cargo build --release
	$(OBJCOPY) -F elf32-i386 target/amd64-kernel/release/kernel kernel
	$(OBJDUMP) -d target/amd64-kernel/release/kernel > kernel.txt

clean:
	$(RUSTUP_RUN) cargo clean
	rm $(OBJS) kernel kernel.txt

.S.o:
	$(CC) $(ASFLAGS) -c -o $@ $<
