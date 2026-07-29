#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdarg.h>
#include <dlfcn.h>
#include <sys/mman.h>
#include <sys/types.h>

static const char *dmi_env(const char *path) {
    static const char prefix[] = "/sys/class/dmi/id/";
    size_t plen = sizeof(prefix) - 1;
    if (strncmp(path, prefix, plen) != 0) return NULL;
    const char *field = path + plen;
    if (*field == '\0' || strchr(field, '/')) return NULL;
    char name[128];
    int n = snprintf(name, sizeof(name), "ZCLOAK_DMI_%s", field);
    if (n < 0 || (size_t)n >= sizeof(name)) return NULL;
    for (char *p = name + 11; *p; p++) *p = (char)toupper((unsigned char)*p);
    return getenv(name);
}

static int is_disk_serial(const char *path) {
    size_t len = strlen(path);
    static const char suf[] = "/serial";
    size_t slen = sizeof(suf) - 1;
    if (len < slen || strcmp(path + len - slen, suf) != 0) return 0;
    if (strncmp(path, "/sys/", 5) != 0) return 0;
    return (strstr(path, "/block/") != NULL) || (strstr(path, "nvme") != NULL);
}

static const char *fake_value_for(const char *path) {
    if (!path) return NULL;
    if (strcmp(path, "/etc/machine-id") == 0 ||
        strcmp(path, "/var/lib/dbus/machine-id") == 0)
        return getenv("ZCLOAK_MACHINE_ID");
    const char *dmi = dmi_env(path);
    if (dmi) return dmi;
    if (is_disk_serial(path)) return getenv("ZCLOAK_DISK_SERIAL");
    return NULL;
}

static int fake_fd(const char *value) {
    int fd = memfd_create("zcloak", 0);
    if (fd < 0) return -1;
    dprintf(fd, "%s\n", value);
    lseek(fd, 0, SEEK_SET);
    return fd;
}

int open(const char *path, int flags, ...) {
    static int (*real_open)(const char *, int, ...) = NULL;
    if (!real_open) real_open = dlsym(RTLD_NEXT, "open");
    const char *fake = fake_value_for(path);
    if (fake) return fake_fd(fake);
    va_list ap; va_start(ap, flags); int mode = va_arg(ap, int); va_end(ap);
    return real_open(path, flags, mode);
}

int open64(const char *path, int flags, ...) {
    static int (*real_open64)(const char *, int, ...) = NULL;
    if (!real_open64) real_open64 = dlsym(RTLD_NEXT, "open64");
    const char *fake = fake_value_for(path);
    if (fake) return fake_fd(fake);
    va_list ap; va_start(ap, flags); int mode = va_arg(ap, int); va_end(ap);
    return real_open64(path, flags, mode);
}

int openat(int dirfd, const char *path, int flags, ...) {
    static int (*real_openat)(int, const char *, int, ...) = NULL;
    if (!real_openat) real_openat = dlsym(RTLD_NEXT, "openat");
    const char *fake = fake_value_for(path);
    if (fake) return fake_fd(fake);
    va_list ap; va_start(ap, flags); int mode = va_arg(ap, int); va_end(ap);
    return real_openat(dirfd, path, flags, mode);
}

int openat64(int dirfd, const char *path, int flags, ...) {
    static int (*real_openat64)(int, const char *, int, ...) = NULL;
    if (!real_openat64) real_openat64 = dlsym(RTLD_NEXT, "openat64");
    const char *fake = fake_value_for(path);
    if (fake) return fake_fd(fake);
    va_list ap; va_start(ap, flags); int mode = va_arg(ap, int); va_end(ap);
    return real_openat64(dirfd, path, flags, mode);
}

FILE *fopen(const char *path, const char *mode) {
    static FILE *(*real_fopen)(const char *, const char *) = NULL;
    if (!real_fopen) real_fopen = dlsym(RTLD_NEXT, "fopen");
    const char *fake = fake_value_for(path);
    if (fake) {
        int fd = fake_fd(fake);
        if (fd >= 0) return fdopen(fd, "r");
    }
    return real_fopen(path, mode);
}

FILE *fopen64(const char *path, const char *mode) {
    static FILE *(*real_fopen64)(const char *, const char *) = NULL;
    if (!real_fopen64) real_fopen64 = dlsym(RTLD_NEXT, "fopen64");
    const char *fake = fake_value_for(path);
    if (fake) {
        int fd = fake_fd(fake);
        if (fd >= 0) return fdopen(fd, "r");
    }
    return real_fopen64(path, mode);
}
