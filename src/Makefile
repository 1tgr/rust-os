DIRS += kernel

DIRS += apps/cairo_demo
DIRS += apps/graphics_client
DIRS += apps/graphics_server
DIRS += apps/hello
DIRS += apps/input

all:
	echo $(DIRS) | xargs -P2 -I_ -n1 make -C _
	make -C boot
