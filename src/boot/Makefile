include ../config.txt

ISO += ../kernel/kernel
ISO += initrd.tar.gz
ISO += menu.lst
ISO += stage2_eltorito

ISO_LAYOUT += kernel=../kernel/kernel
ISO_LAYOUT += initrd.tar.gz
ISO_LAYOUT += boot/grub/menu.lst=menu.lst
ISO_LAYOUT += boot/grub/stage2_eltorito=stage2_eltorito_temp

INITRD += ../target/$(CONFIG_TARGET)/stripped/cairo_demo
INITRD += ../target/$(CONFIG_TARGET)/stripped/graphics_client
INITRD += ../target/$(CONFIG_TARGET)/stripped/graphics_server
INITRD += ../target/$(CONFIG_TARGET)/stripped/hello
INITRD += ../target/$(CONFIG_TARGET)/stripped/input
INITRD += ../target/$(CONFIG_TARGET)/stripped/terminal

ifeq ($(shell uname),Darwin)
TARFLAGS += -s ',.*/,,g'
all: initrd.tar
else
TARFLAGS = --transform 's/.*\///g'
all: bootable.iso
endif

../target/$(CONFIG_TARGET)/stripped:
	mkdir $@

../target/$(CONFIG_TARGET)/stripped/%: ../target/$(CONFIG_TARGET)/release/% | ../target/$(CONFIG_TARGET)/stripped
	x86_64-elf-strip -o $@ $<

initrd.tar: $(INITRD)
	tar $(TARFLAGS) -cf $@ $(INITRD)

initrd.tar.gz: initrd.tar
	gzip -fk $<

bootable.iso: $(ISO)
	cp stage2_eltorito stage2_eltorito_temp
	genisoimage -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4 -boot-info-table -graft-points -o $@ $(ISO_LAYOUT)
	rm stage2_eltorito_temp
