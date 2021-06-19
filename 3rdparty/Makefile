DIRS += newlib
DIRS += zlib
DIRS += libpng
DIRS += freetype
DIRS += pixman
DIRS += cairo

all: tools-arm32-src tools-amd64-src

tools-arm32-binary:
	set -e && for dir in toolchain-binary $(DIRS); do $(MAKE) -C $$dir tools-arm32; done

tools-amd64-binary:
	set -e && for dir in toolchain-binary $(DIRS); do $(MAKE) -C $$dir tools-amd64; done

tools-arm32-src:
	set -e && for dir in binutils-src gcc-src $(DIRS); do $(MAKE) -C $$dir tools-arm32; done

tools-amd64-src:
	set -e && for dir in binutils-src gcc-src $(DIRS); do $(MAKE) -C $$dir tools-amd64; done
