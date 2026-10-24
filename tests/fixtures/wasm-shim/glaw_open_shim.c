/* SQLRDF-27 fcntl ABI bridge, plain-definition variant.
   libduckdb.a was compiled against musl-layout fcntl constants; the
   wasi-libc openat rejects mismatched words with EINVAL (COPY egress).
   Defines open/openat directly (no linker --wrap): DuckDB's undefined
   `open` resolves here; wasi-libc's own open is unused. Calls
   __wasilibc_nocwd_openat_nomode with the translated wasi word. */
#include <errno.h>
#include <stdarg.h>
#include <fcntl.h>

extern int __wasilibc_nocwd_openat_nomode(int, const char *, int, ...);

static int wasi_fcntl_translate(int flags) {
    int out;
    switch (flags & 3) {
    case 1:  out = 0x10000000; break; /* O_WRONLY */
    case 2:  out = 0x0C000000; break; /* O_RDWR   */
    default: out = 0x04000000; break; /* O_RDONLY */
    }
    if (flags & 0100)    out |= 0x1000;     /* O_CREAT    */
    if (flags & 0200)    out |= 0x4000;     /* O_EXCL     */
    if (flags & 01000)   out |= 0x8000;     /* O_TRUNC    */
    if (flags & 0200000) out |= 0x2000;     /* O_DIRECTORY */
    if (flags & 0400000) out |= 0x01000000; /* O_NOFOLLOW */
    if (flags & 02000)   out |= 0x0001;     /* O_APPEND -> fdflags */
    return out;
}

int open(const char *path, int flags, ...) {
    unsigned int mode = 0;
    if (flags & 0100) {
        va_list ap; va_start(ap, flags);
        mode = (unsigned int)va_arg(ap, unsigned int);
        va_end(ap);
    }
    return __wasilibc_nocwd_openat_nomode(AT_FDCWD, path, wasi_fcntl_translate(flags), mode);
}

int openat(int dirfd, const char *path, int flags, ...) {
    unsigned int mode = 0;
    if (flags & 0100) {
        va_list ap; va_start(ap, flags);
        mode = (unsigned int)va_arg(ap, unsigned int);
        va_end(ap);
    }
    return __wasilibc_nocwd_openat_nomode(dirfd, path, wasi_fcntl_translate(flags), mode);
}
