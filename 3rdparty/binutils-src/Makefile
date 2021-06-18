BINUTILS=binutils-2.35
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32: ../target/bin/arm-eabi-ld

tools-amd64: ../target/bin/x86_64-elf-ld

download: src/$(BINUTILS).tar.gz

src/$(BINUTILS).tar.gz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ https://ftp.gnu.org/gnu/binutils/$(@F)

src/$(BINUTILS)/configure: src/$(BINUTILS).tar.gz
	tar -C $(<D) -zxf $<
	touch $@

build/arm32/$(BINUTILS)/Makefile: src/$(BINUTILS)/configure Makefile
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --target=arm-eabi

build/amd64/$(BINUTILS)/Makefile: src/$(BINUTILS)/configure Makefile
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--silent --prefix=$(ROOT)/target --target=x86_64-elf

../target/bin/arm-eabi-ld: build/arm32/$(BINUTILS)/Makefile
	$(MAKE) -C $(<D)
	$(MAKE) -C $(<D) install

../target/bin/x86_64-elf-ld: build/amd64/$(BINUTILS)/Makefile
	$(MAKE) -C $(<D)
	$(MAKE) -C $(<D) install
