FREETYPE=freetype-2.10.2
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32:
	true

tools-amd64: ../target/lib/libfreetype.a

download: src/$(FREETYPE).tar.xz

src/$(FREETYPE).tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ https://download.savannah.gnu.org/releases/freetype/$(@F)

src/$(FREETYPE)/configure: src/$(FREETYPE).tar.xz
	tar -C $(<D) -Jxf $<
	touch $@

build/amd64/$(FREETYPE)/Makefile: src/$(FREETYPE)/configure Makefile ../target/x86_64-elf/lib/libc.a ../target/lib/libz.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --host=x86_64-elf --target=x86_64-elf \
		--enable-shared=no \
		CC=$(ROOT)/target/bin/x86_64-elf-gcc \
        CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -nostartfiles -I $(ROOT)/target/x86_64-elf/include -I $(ROOT)/target/include"

../target/lib/libfreetype.a: build/amd64/$(FREETYPE)/Makefile
	$(MAKE) -C $(<D) all install
