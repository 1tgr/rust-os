ZLIB=zlib-1.2.11
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32:
	true

tools-amd64: ../target/lib/libz.a

download: src/$(ZLIB).tar.gz

src/$(ZLIB).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://zlib.net/$(@F)

src/$(ZLIB)/configure: src/$(ZLIB).tar.gz
	tar -C $(<D) -zxf $<
	touch $@

build/amd64/$(ZLIB)/Makefile: src/$(ZLIB)/configure Makefile ../target/x86_64-elf/lib/libc.a
	mkdir -p $(@D)
	cd $(@D) && \
	env AR=$(ROOT)/target/bin/x86_64-elf-ar \
		CC=$(ROOT)/target/bin/x86_64-elf-gcc \
		CROSS_PREFIX=$(ROOT)/target/bin/x86_64-elf \
        CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -I $(ROOT)/target/x86_64-elf/include" \
        RANLIB=$(ROOT)/target/bin/x86_64-elf-ranlib \
        $(CURDIR)/$< \
		--prefix=$(ROOT)/target --64 --static

../target/lib/libz.a: build/amd64/$(ZLIB)/Makefile
	$(MAKE) -C $(<D) libz.a install
