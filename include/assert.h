#ifdef NDEBUG
# define assert(__e) ((void)0)
#else
# define assert(__e) ((__e) ? (void)0 : __assert(__FILE__, __LINE__, #__e))
#endif

void __assert(const char *, int, const char *);

