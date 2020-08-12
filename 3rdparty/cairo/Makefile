CAIRO=cairo-1.16.0
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32:
	true

tools-amd64: ../target/lib/libcairo.a

download: src/$(CAIRO).tar.xz

src/$(CAIRO).tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://cairographics.org/releases/$(@F)

src/$(CAIRO)/configure: src/$(CAIRO).tar.xz
	tar -C $(<D) -Jxf $<
	touch $@

build/amd64/$(CAIRO)/Makefile: src/$(CAIRO)/configure Makefile ../target/x86_64-elf/lib/libc.a ../target/lib/libfreetype.a ../target/lib/libpixman-1.a ../target/lib/libpng16.a ../target/lib/libz.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --host=x86_64-elf --target=x86_64-elf \
		--enable-ft=yes \
		--enable-fc=no --enable-ps=no --enable-pdf=no --enable-svg=no --enable-script=no --enable-interpreter=no --enable-pthread=no --enable-gobject=no --enable-xlib=no --enable-xlib-xrender=no --enable-xcb=no --enable-xlib-xcb=no --enable-xcb-shm=no \
		CC=$(ROOT)/target/bin/x86_64-elf-gcc \
		CXX=$(ROOT)/target/bin/x86_64-elf-g++ \
        CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -nostartfiles -I $(ROOT)/target/x86_64-elf/include -I $(ROOT)/target/include -DCAIRO_NO_MUTEX=1" \
		PKG_CONFIG_PATH=$(ROOT)/target/lib/pkgconfig

../target/lib/libcairo.a: build/amd64/$(CAIRO)/Makefile
	$(MAKE) -C $(<D)/src all install
