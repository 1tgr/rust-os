DIRS_SRC += kernel

DIRS_SRC += apps/cairo_demo
DIRS_SRC += apps/graphics_client
DIRS_SRC += apps/graphics_server
DIRS_SRC += apps/hello
DIRS_SRC += apps/input
DIRS_SRC += libgraphics

DIRS += $(DIRS_SRC)
DIRS += boot

.PHONY: all test $(DIRS)

all: test

$(DIRS):
	make -C $@

boot: $(DIRS_SRC)

test: $(DIRS)
	python3 test.py qemu-system-x86_64 -no-kvm -display vnc=:1 -no-reboot -kernel kernel/kernel -initrd boot/initrd.tar
