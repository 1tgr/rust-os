ISO += ../../kernel/target/amd64-kernel/stripped/kernel
ISO += initrd.tar.gz
ISO += menu.lst
ISO += stage2_eltorito

ISO_LAYOUT += kernel=../../kernel/target/amd64-kernel/stripped/kernel
ISO_LAYOUT += initrd.tar.gz
ISO_LAYOUT += boot/grub/menu.lst=menu.lst
ISO_LAYOUT += boot/grub/stage2_eltorito=stage2_eltorito_temp

INITRD += ../../target/amd64/stripped/hello
INITRD += ../../target/amd64/stripped/input
INITRD += ../../target/amd64/stripped/ui-demo
INITRD += ../../target/amd64/stripped/ui-server

ifeq ($(shell uname),Darwin)
TARFLAGS += -s ',.*/,,g'
all: initrd.tar
else
TARFLAGS = --transform 's/.*\///g'
all: bootable.iso
endif

../../target/amd64/stripped:
	mkdir $@

../../target/amd64/stripped/%: ../../target/amd64/release/% | ../../target/amd64/stripped
	x86_64-elf-strip -o $@ $<

initrd.tar: $(INITRD)
	tar $(TARFLAGS) -cf $@ $(INITRD)

initrd.tar.gz: initrd.tar
	gzip -fk $<

bootable.iso: $(ISO)
	cp stage2_eltorito stage2_eltorito_temp
	genisoimage -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4 -boot-info-table -graft-points -o $@ $(ISO_LAYOUT)
	rm stage2_eltorito_temp
