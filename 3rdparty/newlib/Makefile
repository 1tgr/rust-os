NEWLIB=newlib-3.3.0
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32: ../target/arm-eabi/lib/libc.a

tools-amd64: ../target/x86_64-elf/lib/libnosys.a ../target/x86_64-elf/lib/libc.a

download: src/$(NEWLIB).tar.gz

src/$(NEWLIB).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ ftp://sourceware.org/pub/newlib/$(@F)

src/$(NEWLIB)/configure: src/$(NEWLIB).tar.gz
	tar -C $(<D) -zxf $<
	touch $@

build/arm32/$(NEWLIB)/newlib/Makefile: src/$(NEWLIB)/configure Makefile ../target/bin/arm-eabi-gcc
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$(<D)/newlib/configure \
		--silent --prefix=$(ROOT)/target --host=arm-eabi --target=arm-eabi --enable-newlib-hw-fp --enable-multilib=no \
		CC=$(ROOT)/target/bin/arm-eabi-gcc \
		CFLAGS="-O -g -I$(CURDIR)/src/$(NEWLIB)/newlib/libc/include -marm -mcpu=arm1176jzf-s -mfloat-abi=hard -mfpu=vfp"

build/amd64/$(NEWLIB)/libgloss/Makefile: src/$(NEWLIB)/configure Makefile ../target/x86_64-elf/lib/libc.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$(<D)/libgloss/configure \
		--silent --prefix=$(ROOT)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(ROOT)/target/bin/x86_64-elf-gcc \
    	CFLAGS="-O -g -mcmodel=kernel -fno-pie"

build/amd64/$(NEWLIB)/newlib/Makefile: src/$(NEWLIB)/configure Makefile ../target/bin/x86_64-elf-gcc
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$(<D)/newlib/configure \
		--silent --prefix=$(ROOT)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(ROOT)/target/bin/x86_64-elf-gcc \
    	CFLAGS="-O -g -I$(CURDIR)/src/$(NEWLIB)/newlib/libc/include -mcmodel=kernel -fno-pie"

../target/arm-eabi/lib/libc.a: build/arm32/$(NEWLIB)/newlib/Makefile
	$(MAKE) -C $(<D) all install

../target/x86_64-elf/lib/libnosys.a: build/amd64/$(NEWLIB)/libgloss/Makefile
	$(MAKE) -C $(<D) all install

../target/x86_64-elf/lib/libc.a: build/amd64/$(NEWLIB)/newlib/Makefile
	$(MAKE) -C $(<D) all install
