#include <stddef.h>

int memcmp(const void *s1, const void *s2, size_t n);

#define	_CONST		const
#define	_PTR		void *
#define	_AND		,
#define	_DEFUN(name, arglist, args)	name(args)
