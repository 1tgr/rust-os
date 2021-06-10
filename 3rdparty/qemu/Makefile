.PHONY: all
ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32: ../target/bin/qemu-system-arm

tools-amd64: ../target/bin/qemu-system-x86_64

download: src/qemu/configure

src/qemu/configure:
	git clone git://git.qemu.org/qemu.git $(@D)
	cd $(@D) && git checkout 7fe7fae8b48e3f9c647fd685e5155ebc8e6fb84d && git submodule update --init dtc

build/qemu/Makefile: src/qemu/configure Makefile
	mkdir -p $(@D)
	cd $(@D) && $(CURDIR)/$< \
		--target-list=arm-softmmu,x86_64-softmmu --prefix=$(ROOT)/target

../target/bin/qemu-system-arm ../target/bin/qemu-system-x86_64 &: build/qemu/Makefile
	$(MAKE) -C $(<D)
	$(MAKE) -C $(<D) install
