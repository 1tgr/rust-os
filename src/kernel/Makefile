include ../config.txt

OBJS = arch/$(CONFIG_TARGET)/start.o arch/$(CONFIG_TARGET)/setjmp.o
RUSTUP_RUN = RUST_TARGET_PATH=../libsyscall/arch rustup run $(CONFIG_RUST_TOOLCHAIN)
CC = gcc
TARGET = x86_64-elf
OBJDUMP = $(TARGET)-objdump
OBJCOPY = $(TARGET)-objcopy
CFLAGS += -m64
CFLAGS += -g -I ../newlib/$(TARGET)/include

.PHONY: all clean

all: $(OBJS)
	$(RUSTUP_RUN) xargo build --target $(CONFIG_TARGET)-kernel --release
	$(OBJCOPY) -SF elf32-i386 target/$(CONFIG_TARGET)-kernel/release/kernel kernel
	$(OBJDUMP) -d target/$(CONFIG_TARGET)-kernel/release/kernel > kernel.txt

clean:
	$(RUSTUP_RUN) xargo clean
	rm $(OBJS) kernel kernel.txt

%.o: %.S
	$(CC) $(CFLAGS) -E $< | $(TARGET)-as $(ASFLAGS) -o $@
