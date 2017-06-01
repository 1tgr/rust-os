DIRS_SRC += apps/cairo_demo
DIRS_SRC += apps/graphics_client
DIRS_SRC += apps/graphics_server
DIRS_SRC += apps/hello
DIRS_SRC += apps/input
DIRS_SRC += kernel

DIRS += $(DIRS_SRC)
DIRS += boot

.PHONY: all $(DIRS)

all: $(DIRS)

$(DIRS):
	make -C $@

boot: $(DIRS_SRC)
