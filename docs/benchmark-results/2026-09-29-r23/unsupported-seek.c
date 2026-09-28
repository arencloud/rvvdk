// Process-local benchmark fault injection, not a filesystem implementation.
#define _GNU_SOURCE
#include <dlfcn.h>
#include <errno.h>
#include <unistd.h>

static int unsupported(int whence) {
    if (whence == SEEK_DATA || whence == SEEK_HOLE) {
        errno = EINVAL;
        return 1;
    }
    return 0;
}

off_t lseek(int fd, off_t offset, int whence) {
    if (unsupported(whence)) return -1;
    off_t (*next)(int, off_t, int) = dlsym(RTLD_NEXT, "lseek");
    return next(fd, offset, whence);
}

off64_t lseek64(int fd, off64_t offset, int whence) {
    if (unsupported(whence)) return -1;
    off64_t (*next)(int, off64_t, int) = dlsym(RTLD_NEXT, "lseek64");
    return next(fd, offset, whence);
}
