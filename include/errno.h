extern int * __error(void);
#define errno (*__error())

#define	ENOMEM		12		/* Cannot allocate memory */

