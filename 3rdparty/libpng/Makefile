LIBPNG=libpng-1.6.37
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32:
	true

tools-amd64: ../target/lib/libpng16.a

download: src/$(LIBPNG).tar.xz

src/$(LIBPNG).tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://prdownloads.sourceforge.net/libpng/$(@F)?download

src/$(LIBPNG)/configure: src/$(LIBPNG).tar.xz
	tar -C $(<D) -Jxf $<
	touch $@

build/amd64/$(LIBPNG)/Makefile: src/$(LIBPNG)/configure Makefile ../target/x86_64-elf/lib/libc.a ../target/lib/libz.a ../target/x86_64-elf/lib/libnosys.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(ROOT)/target/bin/x86_64-elf-gcc \
		CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -nostartfiles -I $(ROOT)/target/x86_64-elf/include -I $(ROOT)/target/include" \
		CPPFLAGS="-I $(ROOT)/target/x86_64-elf/include -I $(ROOT)/target/include" \
		LDFLAGS="-L$(ROOT)/target/lib" \
		LIBS="-lc -lnosys"

../target/lib/libpng16.a: build/amd64/$(LIBPNG)/Makefile
	$(MAKE) -C $(<D) install-libLTLIBRARIES install-data-am
