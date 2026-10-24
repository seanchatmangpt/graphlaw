/* SQLRDF-27 fcntl ABI bridge (linker --wrap variant, sole __wrap provider). */
#include <errno.h>
#include <stdarg.h>

extern int __real_open(const char *path, int flags, ...);
extern int __real_openat(int dirfd, const char *path, int flags, ...);

static int wasi_fcntl_translate(int flags) {
    int out;
    switch (flags & 3) {
    case 1:  out = 0x10000000; break; /* O_WRONLY */
    case 2:  out = 0x0C000000; break; /* O_RDWR   */
    default: out = 0x04000000; break; /* O_RDONLY */
    }
    if (flags & 0100)    out |= 0x1000;
    if (flags & 0200)    out |= 0x4000;
    if (flags & 01000)   out |= 0x8000;
    if (flags & 0200000) out |= 0x2000;
    if (flags & 0400000) out |= 0x01000000;
    if (flags & 02000)   out |= 0x0001;
    return out;
}

int __wrap_open(const char *path, int flags, ...) {
    unsigned int mode = 0;
    if (flags & 0100) {
        va_list ap; va_start(ap, flags);
        mode = (unsigned int)va_arg(ap, unsigned int);
        va_end(ap);
    }
    return __real_open(path, wasi_fcntl_translate(flags), mode);
}

int __wrap_openat(int dirfd, const char *path, int flags, ...) {
    unsigned int mode = 0;
    if (flags & 0100) {
        va_list ap; va_start(ap, flags);
        mode = (unsigned int)va_arg(ap, unsigned int);
        va_end(ap);
    }
    return __real_openat(dirfd, path, wasi_fcntl_translate(flags), mode);
}
