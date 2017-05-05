include ../config.txt

OBJS = arch/$(CONFIG_TARGET)/start.o arch/$(CONFIG_TARGET)/setjmp.o
RUSTUP_RUN = RUST_TARGET_PATH=../libsyscall/arch rustup run $(CONFIG_RUST_TOOLCHAIN)
CC = gcc
OBJDUMP = $(TARGET)-objdump
OBJCOPY = $(TARGET)-objcopy
ASFLAGS += -m64
ASFLAGS += -g -I ../newlib/$(TARGET)/include

.PHONY: all clean

all: $(OBJS)
	$(RUSTUP_RUN) xargo build --target $(CONFIG_TARGET)-kernel --release
	$(OBJCOPY) -F elf32-i386 target/$(CONFIG_TARGET)-kernel/release/kernel kernel
	$(OBJDUMP) -d target/$(CONFIG_TARGET)-kernel/release/kernel > kernel.txt

clean:
	$(RUSTUP_RUN) xargo clean
	rm $(OBJS) kernel kernel.txt

.S.o:
	$(CC) $(ASFLAGS) -c -o $@ $<
