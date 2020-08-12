DIRS += binutils
DIRS += gcc
DIRS += newlib
DIRS += zlib
DIRS += libpng
DIRS += freetype
DIRS += pixman
DIRS += cairo
DIRS += qemu

.PHONY: all tools-arm32 tools-amd64 download

all: tools-arm32 tools-amd64

tools-arm32:
	set -e && for dir in $(DIRS); do $(MAKE) -C $$dir tools-arm32; done

tools-amd64:
	set -e && for dir in $(DIRS); do $(MAKE) -C $$dir tools-amd64; done

download:
	set -e && for dir in $(DIRS); do $(MAKE) -C $$dir download; done
