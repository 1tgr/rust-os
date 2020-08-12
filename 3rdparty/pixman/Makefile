PIXMAN=pixman-0.40.0
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32:
	true

tools-amd64: ../target/lib/libpixman-1.a

download: src/$(PIXMAN).tar.gz

src/$(PIXMAN).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ http://cairographics.org/releases/$(@F)

src/$(PIXMAN)/configure: src/$(PIXMAN).tar.gz
	tar -C $(<D) -zxf $<
	touch $@

build/amd64/$(PIXMAN)/Makefile: src/$(PIXMAN)/configure Makefile ../target/x86_64-elf/lib/libc.a ../target/lib/libz.a
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --host=x86_64-elf --target=x86_64-elf \
		CC=$(ROOT)/target/bin/x86_64-elf-gcc \
      	CFLAGS="-O -g -mcmodel=kernel -fno-pie -ffreestanding -nostartfiles -I $(ROOT)/target/x86_64-elf/include -I $(ROOT)/target/include -DPIXMAN_NO_TLS" \
      	LDFLAGS="-L$(ROOT)/target/lib"

../target/lib/libpixman-1.a: build/amd64/$(PIXMAN)/Makefile
	$(MAKE) -C $(<D)/pixman all install
