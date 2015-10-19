CAIRO=cairo-1.14.2
LIBPNG=libpng-1.6.18
NEWLIB=newlib-2.2.0.20150924
PIXMAN=pixman-0.32.8
ZLIB=zlib-1.2.8

ROOT=$(shell cd "$(CURDIR)/.." && pwd)
SRC=$(ROOT)/src

DOWNLOADS += $(CAIRO).tar.xz
DOWNLOADS += $(LIBPNG).tar.xz
DOWNLOADS += $(NEWLIB).tar.gz
DOWNLOADS += $(PIXMAN).tar.gz
DOWNLOADS += $(ZLIB).tar.gz

DIRS += $(CAIRO)/
DIRS += $(NEWLIB)/
DIRS += $(PIXMAN)/
DIRS += $(LIBPNG)/
DIRS += $(ZLIB)/

MAKEFILES += $(CAIRO)/Makefile
MAKEFILES += $(NEWLIB)/newlib/Makefile
MAKEFILES += $(PIXMAN)/pixman/Makefile
MAKEFILES += $(LIBPNG)/Makefile
MAKEFILES += $(ZLIB)/Makefile

LIBS += $(SRC)/cairo/lib/libcairo.a
LIBS += $(SRC)/cairo/lib/libpixman-1.a
LIBS += $(SRC)/cairo/lib/libpng16.a
LIBS += $(SRC)/cairo/lib/libz.a
LIBS += $(SRC)/newlib/x86_64-elf/lib/libc.a

.PHONY: all clean clean_libs clean_dirs clean_downloads download dirs configure

all: $(LIBS)

clean_libs:
	rm $(LIBS)

clean_dirs:
	rm -r $(DIRS)

clean_downloads:
	rm $(DOWNLOADS)

clean: clean_downloads clean_dirs

download: $(DOWNLOADS)
dirs: $(DIRS)
configure: $(MAKEFILES)

$(CAIRO).tar.xz:
	curl -Lo $@ http://cairographics.org/releases/$@

$(LIBPNG).tar.xz:
	curl -Lo $@ http://prdownloads.sourceforge.net/libpng/$@?download

$(NEWLIB).tar.gz:
	curl -Lo $@ ftp://sourceware.org/pub/newlib/$@

$(PIXMAN).tar.gz:
	curl -Lo $@ http://cairographics.org/releases/$@

$(ZLIB).tar.gz:
	curl -Lo $@ http://zlib.net/$@

$(CAIRO)/: $(CAIRO).tar.xz
	tar -Jxf $<

$(PIXMAN)/: $(PIXMAN).tar.gz
	tar -zxf $<

$(LIBPNG)/: $(LIBPNG).tar.xz
	tar -Jxf $<

$(NEWLIB)/: $(NEWLIB).tar.gz
	tar -zxf $<

$(ZLIB)/: $(ZLIB).tar.gz
	tar -zxf $<

$(CAIRO)/Makefile: $(CAIRO)/ $(SRC)/newlib/x86_64-elf/lib/libc.a $(SRC)/cairo/lib/libpixman-1.a $(SRC)/cairo/lib/libpng16.a $(SRC)/cairo/lib/libz.a
	cd $(CAIRO) && \
	CFLAGS="-O -g -mcmodel=kernel -ffreestanding -I $(SRC)/newlib/x86_64-elf/include -I $(SRC)/cairo/include -DCAIRO_NO_MUTEX=1" \
	PKG_CONFIG_PATH=$(SRC)/cairo/lib/pkgconfig \
	./configure --prefix=$(SRC)/cairo --host=x86_64-elf --enable-ps=no --enable-ft=no --enable-pdf=no --enable-svg=no --enable-script=no --enable-interpreter=no --enable-pthread=no

$(LIBPNG)/Makefile: $(LIBPNG)/ $(SRC)/newlib/x86_64-elf/lib/libc.a $(SRC)/cairo/lib/libz.a
	cd $(LIBPNG) && \
	CFLAGS="-O -g -mcmodel=kernel -ffreestanding -I $(SRC)/newlib/x86_64-elf/include -I $(SRC)/cairo/include" \
	CPPFLAGS="-I $(SRC)/newlib/x86_64-elf/include -I $(SRC)/cairo/include" \
	LDFLAGS="-L$(SRC)/cairo/lib" \
	./configure --prefix=$(SRC)/cairo --host=x86_64-elf

$(NEWLIB)/newlib/Makefile: $(NEWLIB)/
	cd $(NEWLIB)/newlib && \
	CFLAGS="-O -g -mcmodel=kernel" \
	./configure --prefix=$(SRC)/newlib --host=x86_64-elf

$(PIXMAN)/pixman/Makefile: $(PIXMAN)/ $(SRC)/newlib/x86_64-elf/lib/libc.a
	cd $(PIXMAN) && \
	CFLAGS="-O -g -mcmodel=kernel -ffreestanding -I $(SRC)/newlib/x86_64-elf/include -I $(SRC)/cairo/include -DPIXMAN_NO_TLS" \
	./configure --prefix=$(SRC)/cairo --host=x86_64-elf

$(ZLIB)/Makefile: $(ZLIB)/ $(SRC)/newlib/x86_64-elf/lib/libc.a
	cd $(ZLIB) && \
	CFLAGS="-O -g -mcmodel=kernel -ffreestanding -I $(SRC)/newlib/x86_64-elf/include" \
	./configure --prefix=$(SRC)/cairo --64

$(SRC)/cairo/lib/libcairo.a: $(SRC)/newlib/x86_64-elf/lib/libc.a $(SRC)/cairo/lib/libpixman-1.a $(SRC)/cairo/lib/libz.a $(CAIRO)/Makefile
	make -C $(CAIRO)/src all install

$(SRC)/cairo/lib/libpixman-1.a: $(SRC)/newlib/x86_64-elf/lib/libc.a $(PIXMAN)/pixman/Makefile
	make -C $(PIXMAN)/pixman all install

$(SRC)/cairo/lib/libpng16.a: $(SRC)/newlib/x86_64-elf/lib/libc.a $(SRC)/cairo/lib/libz.a $(LIBPNG)/Makefile
	make -C $(LIBPNG) .libs/libpng16.a install-libLTLIBRARIES install-data-am

$(SRC)/cairo/lib/libz.a: $(SRC)/newlib/x86_64-elf/lib/libc.a $(ZLIB)/Makefile
	make -C $(ZLIB) libz.a install

$(SRC)/newlib/x86_64-elf/lib/libc.a: $(NEWLIB)/newlib/Makefile
	make -C $(NEWLIB)/newlib all install
