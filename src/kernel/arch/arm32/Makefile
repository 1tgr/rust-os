OBJS = start.o setjmp.o
RUSTUP_RUN = RUST_TARGET_PATH=${CURDIR}/../../../syscall/arch RUSTFLAGS=-Dwarnings
CC = gcc
TARGET = arm-none-eabi
OBJDUMP = $(TARGET)-objdump
CFLAGS += -g -I ../../../newlib/$(TARGET)/include

.PHONY: all clean

all: $(OBJS)
	$(RUSTUP_RUN) xargo build --target arm32 --release
	$(OBJDUMP) -S ../../target/arm32/release/kernel > kernel.txt

clean:
	$(RUSTUP_RUN) xargo clean
	rm $(OBJS) kernel.txt

%.o: %.S
	$(CC) $(CFLAGS) -E $< | $(TARGET)-as $(ASFLAGS) -o $@
