ISO += ../kernel/kernel
ISO += initrd.tar.gz
ISO += menu.lst
ISO += stage2_eltorito

ISO_LAYOUT += kernel=../kernel/kernel
ISO_LAYOUT += initrd.tar.gz
ISO_LAYOUT += boot/grub/menu.lst=menu.lst
ISO_LAYOUT += boot/grub/stage2_eltorito=stage2_eltorito_temp

# INITRD += ../apps/cairo_demo/cairo_demo
# INITRD += ../apps/graphics_client/graphics_client
# INITRD += ../apps/graphics_server/graphics_server
INITRD += ../apps/hello/target/amd64/release/hello
INITRD += ../apps/input/target/amd64/release/input

all: bootable.iso

initrd.tar: $(INITRD)
	tar --transform 's/.*\///g' -cf $@ $<

initrd.tar.gz: initrd.tar
	gzip -fk $<

bootable.iso: $(ISO)
	cp stage2_eltorito stage2_eltorito_temp
	genisoimage -R -b boot/grub/stage2_eltorito -no-emul-boot -boot-load-size 4 -boot-info-table -graft-points -o $@ $(ISO_LAYOUT)
	rm stage2_eltorito_temp
