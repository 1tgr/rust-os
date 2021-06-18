ROOT=$(abspath $(CURDIR)/..)

.PHONY: all tools-arm32 tools-amd64

all: tools-arm32 tools-amd64

tools-arm32: ../target/bin/arm-eabi-ld

tools-amd64: ../target/bin/x86_64-elf-ld

src/arm-eabi-10.2.0-Linux-x86_64.tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ https://newos.org/toolchains/$(@F)

src/x86_64-elf-10.2.0-Linux-x86_64.tar.xz:
	mkdir -p $(@D)
	curl --silent --fail -Lo $@ https://newos.org/toolchains/$(@F)

../target/bin/arm-eabi-ld: src/arm-eabi-10.2.0-Linux-x86_64.tar.xz
	mkdir -p $(@D)
	tar -C ../target --strip-components=1 -Jxf $(abspath $<)

../target/bin/x86_64-elf-ld: src/x86_64-elf-10.2.0-Linux-x86_64.tar.xz
	mkdir -p $(@D)
	tar -C ../target --strip-components=1 -Jxf $(abspath $<)
