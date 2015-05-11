
# CONFIG: Architecture to build for
ARCH ?= amd64

# Toolchain commands (can be overridden)
RUSTC ?= rustc
ifeq ($(ARCH),amd64)
    LD := x86_64-elf-ld
    AS := x86_64-elf-as
    OBJDUMP := x86_64-elf-objdump
    OBJCOPY := x86_64-elf-objcopy
    AR := x86_64-elf-ar
	CC := clang -target x86_64-elf
else ifeq ($(ARCH),x86)
    LD := i586-elf-ld
    AS := i586-elf-as
    OBJDUMP := i586-elf-objdump
    AR := i586-elf-ar
	CC := clang -target i586-elf
else
    $(error Unknown architecture $(ARCH))
endif

# Object directory
OBJDIR := .obj/$(ARCH)/

LINKSCRIPT := Kernel/arch/$(ARCH)/link.ld
TARGETSPEC := Kernel/arch/$(ARCH)/target.json
# Compiler Options
LINKFLAGS := -T $(LINKSCRIPT)
LINKFLAGS += -Map $(OBJDIR)map.txt
LINKFLAGS += --gc-sections
LINKFLAGS += -z max-page-size=0x1000

RUSTFLAGS := -O --cfg arch__$(ARCH) --target=$(TARGETSPEC)
# - amd64 needs to be set to use soft floating point
ifeq ($(ARCH),amd64)
RUSTFLAGS += -C soft-float
CCFLAGS += -mcmodel=kernel
endif

# Objects
LIBCORE := $(OBJDIR)libcore.rlib
OBJS := start.o kernel.o liballoc.rlib libcore.rlib liblibc.rlib libc.a
OBJS := $(OBJS:%=$(OBJDIR)%)
BIN := kernel.$(ARCH).bin

.PHONY: all clean

all: kernel.$(ARCH).bin

clean:
	$(RM) -rf $(BIN) $(BIN).dsm $(OBJDIR)

# Final link command
$(BIN): $(OBJS) Kernel/arch/$(ARCH)/link.ld
	$(LD) -o $@ $(LINKFLAGS) $(OBJS)
	$(OBJDUMP) -S $@ > $@.dsm
ifeq ($(ARCH),amd64)
	@mv $@ $@.elf64
	@$(OBJCOPY) $@.elf64 -F elf32-i386 $@
endif

$(OBJDIR)libc/%.o: libc/%.S
	@mkdir -p $(dir $@)
	$(CC) $(ASFLAGS) -c -o $@ $< -D__USER_LABEL_PREFIX__=

$(OBJDIR)libc/%.o: libc/%.c
	@mkdir -p $(dir $@)
	$(CC) $(CCFLAGS) -c -o $@ $< -DHAVE_MMAP=0 -DLACKS_UNISTD_H -DLACKS_SYS_PARAM_H -nostdinc -Iinclude

.obj/amd64/libc.a: $(OBJDIR)libc/malloc.o .obj/amd64/libc/arch/amd64/memcpy.o .obj/amd64/libc/arch/amd64/memset.o
	$(AR) $(ARFLAGS) -o $@ $(OBJDIR)libc/malloc.o .obj/amd64/libc/arch/amd64/memcpy.o .obj/amd64/libc/arch/amd64/memset.o

$(OBJDIR)liballoc.rlib: liballoc/lib.rs $(OBJDIR)libcore.rlib $(OBJDIR)liblibc.rlib $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --cfg external_funcs --crate-type=lib --emit=link,dep-info $< --extern core=$(OBJDIR)libcore.rlib  --extern libc=$(OBJDIR)liblibc.rlib

$(OBJDIR)liblibc.rlib: liblibc/lib.rs $(OBJDIR)libcore.rlib $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --crate-type=lib --emit=link,dep-info $< --extern core=$(OBJDIR)libcore.rlib

$(OBJDIR)libcore.rlib: libcore/lib.rs $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --crate-type=lib --emit=link,dep-info $<

# Compile rust kernel object
$(OBJDIR)kernel.o: Kernel/main.rs $(OBJDIR)libcore.rlib $(OBJDIR)liblibc.rlib $(OBJDIR)liballoc.rlib $(TARGETSPEC)
	@mkdir -p $(dir $@)
	$(RUSTC) $(RUSTFLAGS) -o $@ --emit=obj,dep-info -L $(OBJDIR) $< --extern core=$(OBJDIR)libcore.rlib --extern libc=$(OBJDIR)liblibc.rlib --extern alloc=$(OBJDIR)liballoc.rlib

# Compile architecture's assembly stub
$(OBJDIR)start.o: Kernel/arch/$(ARCH)/start.S 
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $<

# Include dependency files
-include $(OBJDIR)liballoc.d $(OBJDIR)libcore.d $(OBJDIR)kernel.d $(OBJDIR)start.d
