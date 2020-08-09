BINUTILS=binutils-2.35
CAIRO=cairo-1.16.0
GCC=gcc-10.2.0
LIBPNG=libpng-1.6.37
NEWLIB=newlib-3.3.0
PIXMAN=pixman-0.40.0
QEMU=qemu
ZLIB=zlib-1.2.11

DOWNLOADS += src/$(BINUTILS).tar.gz
DOWNLOADS += src/$(CAIRO).tar.xz
DOWNLOADS += src/$(GCC).tar.xz
DOWNLOADS += src/$(LIBPNG).tar.xz
DOWNLOADS += src/$(NEWLIB).tar.gz
DOWNLOADS += src/$(PIXMAN).tar.gz
DOWNLOADS += src/$(ZLIB).tar.gz

LIBS += target/arm-none-eabi/lib/libc.a
LIBS += target/bin/arm-none-eabi-ld
LIBS += target/bin/qemu-system-arm
LIBS += target/bin/qemu-system-x86_64
LIBS += target/bin/x86_64-elf-ld
LIBS += target/lib/libcairo.a
LIBS += target/lib/libpixman-1.a
LIBS += target/lib/libpng16.a
LIBS += target/lib/libz.a
LIBS += target/x86_64-elf/lib/libc.a

.ONESHELL:

.PHONY: all tools tools-arm32 tools-amd64 clean clean_libs clean_dirs clean_downloads download dirs configure

all: $(LIBS)

download: $(DOWNLOADS)

src/$(BINUTILS).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ https://ftp.gnu.org/gnu/binutils/$(@F)

src/$(CAIRO).tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://cairographics.org/releases/$(@F)

src/$(GCC).tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://ftp.gnu.org/gnu/gcc/gcc-10.2.0/$(@F)

src/$(LIBPNG).tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://prdownloads.sourceforge.net/libpng/$(@F)?download

src/$(NEWLIB).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ ftp://sourceware.org/pub/newlib/$(@F)

src/$(PIXMAN).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://cairographics.org/releases/$(@F)

src/$(ZLIB).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://zlib.net/$(@F)

src/$(BINUTILS)/configure: src/$(BINUTILS).tar.gz
	tar -C $(<D) --touch -zxf $<

src/$(CAIRO)/configure: src/$(CAIRO).tar.xz
	tar -C $(<D) --touch -Jxf $<

src/$(GCC)/configure: src/$(GCC).tar.xz
	tar -C $(<D) --touch -Jxf $<

src/$(PIXMAN)/configure: src/$(PIXMAN).tar.gz
	tar -C $(<D) --touch -zxf $<

src/$(QEMU)/configure:
	git clone git://git.qemu-project.org/qemu.git $(@D)
	cd $(@D) && git submodule update --init dtc

src/$(LIBPNG)/configure: src/$(LIBPNG).tar.xz
	tar -C $(<D) --touch -Jxf $<

src/$(NEWLIB)/configure: src/$(NEWLIB).tar.gz
	tar -C $(<D) --touch -zxf $<

src/$(ZLIB)/configure: src/$(ZLIB).tar.gz
	tar -C $(<D) --touch -zxf $<

build/arm32/$(BINUTILS)/Makefile: src/$(BINUTILS)/configure
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(CURDIR)/target --target=arm-none-eabi

build/amd64/$(BINUTILS)/Makefile: src/$(BINUTILS)/configure
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(CURDIR)/target --target=x86_64-elf

build/amd64/$(CAIRO)/Makefile: src/$(CAIRO)/configure target/x86_64-elf/lib/libc.a target/lib/libpixman-1.a target/lib/libpng16.a target/lib/libz.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(CURDIR)/target --host=x86_64-elf --target=x86_64-elf \
		--enable-ps=no --enable-ft=no --enable-pdf=no --enable-svg=no --enable-script=no --enable-interpreter=no --enable-pthread=no --enable-gobject=no --enable-xlib=no --enable-xlib-xrender=no --enable-xcb=no --enable-xlib-xcb=no --enable-xcb-shm=no \
		CC=$(CURDIR)/target/bin/x86_64-elf-gcc \
		CXX=$(CURDIR)/target/bin/x86_64-elf-g++ \
        CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -nostartfiles -I $(CURDIR)/target/x86_64-elf/include -I $(CURDIR)/target/include -DCAIRO_NO_MUTEX=1" \
		PKG_CONFIG_PATH=$(CURDIR)/target/lib/pkgconfig

build/arm32/$(GCC)/Makefile: src/$(GCC)/configure
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(CURDIR)/target --target=arm-none-eabi --disable-nls --enable-languages=c,c++ --without-headers

build/amd64/$(GCC)/Makefile: src/$(GCC)/configure
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(CURDIR)/target --target=x86_64-elf --disable-nls --enable-languages=c,c++ --without-headers

build/$(QEMU)/Makefile: src/$(QEMU)/configure
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--target-list=arm-softmmu,x86_64-softmmu --prefix=$(CURDIR)/target

build/amd64/$(LIBPNG)/Makefile: src/$(LIBPNG)/configure target/x86_64-elf/lib/libc.a target/lib/libz.a target/x86_64-elf/lib/libgloss.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(CURDIR)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(CURDIR)/target/bin/x86_64-elf-gcc \
		CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -nostartfiles -I $(CURDIR)/target/x86_64-elf/include -I $(CURDIR)/target/include" \
		CPPFLAGS="-I $(CURDIR)/target/x86_64-elf/include -I $(CURDIR)/target/include" \
		LDFLAGS="-L$(CURDIR)/target/lib" \
		LIBS="-lc -lnosys"

build/arm32/$(NEWLIB)/newlib/Makefile: src/$(NEWLIB)/configure target/bin/arm-none-eabi-gcc
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$(<D)/newlib/configure \
		--silent --prefix=$(CURDIR)/target --host=arm-none-eabi --target=arm-none-eabi --enable-newlib-hw-fp --enable-multilib=no \
		CC=$(CURDIR)/target/bin/arm-none-eabi-gcc \
		CFLAGS="-O -g -I $(CURDIR)/src/$(NEWLIB)/newlib/libc/include -marm -mcpu=arm1176jzf-s -mfloat-abi=hard -mfpu=vfp"

build/amd64/$(NEWLIB)/libgloss/Makefile: src/$(NEWLIB)/configure target/bin/x86_64-elf-gcc
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$(<D)/libgloss/configure \
		--silent --prefix=$(CURDIR)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(CURDIR)/target/bin/x86_64-elf-gcc \
    	CFLAGS="-O -g -mcmodel=kernel -fno-pie"

build/amd64/$(NEWLIB)/newlib/Makefile: src/$(NEWLIB)/configure target/bin/x86_64-elf-gcc
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$(<D)/newlib/configure \
		--silent --prefix=$(CURDIR)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(CURDIR)/target/bin/x86_64-elf-gcc \
    	CFLAGS="-O -g -I $(CURDIR)/src/$(NEWLIB)/newlib/libc/include -mcmodel=kernel -fno-pie"

build/amd64/$(PIXMAN)/Makefile: src/$(PIXMAN)/configure target/x86_64-elf/lib/libc.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(CURDIR)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(CURDIR)/target/bin/x86_64-elf-gcc \
      	CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -nostartfiles -I $(CURDIR)/target/x86_64-elf/include -I $(CURDIR)/target/include -DPIXMAN_NO_TLS" \
      	LDFLAGS="-L$(CURDIR)/target/lib"

build/amd64/$(ZLIB)/Makefile: src/$(ZLIB)/configure target/x86_64-elf/lib/libc.a
	mkdir -p $(@D)
	cd $(@D) && \
	env AR=$(CURDIR)/target/bin/x86_64-elf-ar \
		CC=$(CURDIR)/target/bin/x86_64-elf-gcc \
		CROSS_PREFIX=$(CURDIR)/target/bin/x86_64-elf \
        CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -I $(CURDIR)/target/x86_64-elf/include" \
        RANLIB=$(CURDIR)/target/bin/x86_64-elf-ranlib \
        $(CURDIR)/$< \
		--prefix=$(CURDIR)/target --64 --static

target/bin/arm-none-eabi-gcc: build/arm32/$(GCC)/Makefile target/bin/arm-none-eabi-ld
	$(MAKE) -C $(<D) all-gcc
	$(MAKE) -C $(<D) all-target-libgcc
	$(MAKE) -C $(<D) install-gcc
	$(MAKE) -C $(<D) install-target-libgcc

target/bin/arm-none-eabi-ld: build/arm32/$(BINUTILS)/Makefile
	$(MAKE) -C $(<D)
	$(MAKE) -C $(<D) install

target/bin/x86_64-elf-gcc: build/amd64/$(GCC)/Makefile target/bin/x86_64-elf-ld
	$(MAKE) -C $(<D) all-gcc
	$(MAKE) -C $(<D) all-target-libgcc
	$(MAKE) -C $(<D) install-gcc
	$(MAKE) -C $(<D) install-target-libgcc

target/bin/x86_64-elf-ld: build/amd64/$(BINUTILS)/Makefile
	$(MAKE) -C $(<D)
	$(MAKE) -C $(<D) install

target/bin/qemu-system-arm target/bin/qemu-system-x86_64 &: build/$(QEMU)/Makefile
	$(MAKE) -C $(<D)
	$(MAKE) -C $(<D) install

target/lib/libcairo.a: build/amd64/$(CAIRO)/Makefile target/x86_64-elf/lib/libc.a target/lib/libpixman-1.a target/lib/libz.a
	$(MAKE) -C $(<D)/src all install

target/lib/libpixman-1.a: build/amd64/$(PIXMAN)/Makefile target/x86_64-elf/lib/libc.a target/lib/libz.a
	$(MAKE) -C $(<D)/pixman all install

target/lib/libpng16.a: build/amd64/$(LIBPNG)/Makefile target/x86_64-elf/lib/libc.a target/lib/libz.a
	$(MAKE) -C $(<D) install-libLTLIBRARIES install-data-am

target/lib/libz.a: build/amd64/$(ZLIB)/Makefile target/x86_64-elf/lib/libc.a
	$(MAKE) -C $(<D) libz.a install

target/arm-none-eabi/lib/libc.a: build/arm32/$(NEWLIB)/newlib/Makefile
	$(MAKE) -C $(<D) all install

target/x86_64-elf/lib/libgloss.a: build/amd64/$(NEWLIB)/libgloss/Makefile
	$(MAKE) -C $(<D) all install

target/x86_64-elf/lib/libc.a: build/amd64/$(NEWLIB)/newlib/Makefile
	$(MAKE) -C $(<D) all install

tools: tools-arm32 tools-amd64

tools-arm32: target/bin/arm-none-eabi-ld target/bin/arm-none-eabi-gcc target/arm-none-eabi/lib/libc.a target/bin/qemu-system-arm

tools-amd64: target/bin/x86_64-elf-ld target/bin/x86_64-elf-gcc target/x86_64-elf/lib/libc.a target/lib/libcairo.a target/bin/qemu-system-x86_64
