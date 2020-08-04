OBJS = start.o setjmp.o
RUSTUP_RUN = RUST_TARGET_PATH=${CURDIR}/../../../syscall/arch RUSTFLAGS=-Dwarnings
CFLAGS += -m64
CFLAGS += -g -I ../../../newlib/x86_64-elf/include

.PHONY: all clean

all: $(OBJS)
	$(RUSTUP_RUN) xargo build --target amd64-kernel --release
	mkdir -p ../../target/amd64-kernel/stripped
	x86_64-elf-objcopy -SF elf32-i386 ../../target/amd64-kernel/release/kernel ../../target/amd64-kernel/stripped/kernel
	x86_64-elf-objdump -S ../../target/amd64-kernel/release/kernel > kernel.txt

clean:
	$(RUSTUP_RUN) xargo clean
	rm $(OBJS) kernel kernel.txt

%.o: %.S
	x86_64-elf-gcc $(CFLAGS) -E $< | x86_64-elf-as $(ASFLAGS) -o $@
