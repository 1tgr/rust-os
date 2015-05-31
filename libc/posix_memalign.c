#include <stddef.h>
#include <errno.h>
#include "malloc.h"

int posix_memalign(void **memptr, size_t alignment, size_t size) {
    *memptr = malloc(size);
    return *memptr ? 0 : errno;
}
