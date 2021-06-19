GCC=gcc-10.2.0
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32: ../target/bin/arm-eabi-gcc

tools-amd64: ../target/bin/x86_64-elf-gcc

download: src/$(GCC).tar.xz

src/$(GCC).tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://ftp.gnu.org/gnu/gcc/gcc-10.2.0/$(@F)

src/$(GCC)/configure: src/$(GCC).tar.xz
	tar -C $(<D) -Jxf $<
	touch $@

build/arm32/$(GCC)/Makefile: src/$(GCC)/configure Makefile
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --target=arm-eabi --disable-nls --enable-languages=c,c++ --without-headers

build/amd64/$(GCC)/Makefile: src/$(GCC)/configure Makefile
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --target=x86_64-elf --disable-nls --enable-languages=c,c++ --without-headers

../target/bin/arm-eabi-gcc: build/arm32/$(GCC)/Makefile ../target/bin/arm-eabi-ld
	$(MAKE) -C $(<D) all-gcc
	$(MAKE) -C $(<D) all-target-libgcc
	$(MAKE) -C $(<D) install-gcc
	$(MAKE) -C $(<D) install-target-libgcc

../target/bin/x86_64-elf-gcc: build/amd64/$(GCC)/Makefile ../target/bin/x86_64-elf-ld
	$(MAKE) -C $(<D) all-gcc
	$(MAKE) -C $(<D) all-target-libgcc
	$(MAKE) -C $(<D) install-gcc
	$(MAKE) -C $(<D) install-target-libgcc
