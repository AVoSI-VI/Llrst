/* Host stat(2), dirent, and (on Apple) libc calls whose numbers differ from Linux.
 * Field offsets come from the platform headers. */
#include <dirent.h>
#include <pthread.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>
#ifdef __APPLE__
#include <mach-o/dyld.h>
#endif

int64_t llr_stat_mtime(const char *path, int64_t *out) {
    struct stat st;
    if (stat(path, &st) != 0) {
        return -1;
    }
    *out = (int64_t)st.st_mtime;
    return 0;
}

int64_t llr_stat_size(const char *path, int64_t *out) {
    struct stat st;
    if (stat(path, &st) != 0) {
        return -1;
    }
    *out = (int64_t)st.st_size;
    return 0;
}

const char *llr_readdir_name(const void *ent) {
    if (ent == 0) {
        return 0;
    }
    return ((const struct dirent *)ent)->d_name;
}

/* Bytes written, not including the NUL, or -1 when the buffer is too small. */
int64_t llr_exe_path(char *buf, uint64_t n) {
    if (buf == 0 || n == 0) {
        return -1;
    }
#ifdef __APPLE__
    uint32_t sz = n > 0xffffffffu ? 0xffffffffu : (uint32_t)n;
    if (_NSGetExecutablePath(buf, &sz) != 0) {
        return -1;
    }
    return (int64_t)strlen(buf);
#else
    ssize_t got = readlink("/proc/self/exe", buf, (size_t)n);
    if (got < 0 || (uint64_t)got >= n) {
        return -1;
    }
    buf[got] = 0;
    return (int64_t)got;
#endif
}

struct llr_frame {
    struct llr_frame *prev;
    const char *name;
};

static pthread_key_t trace_key;
static pthread_once_t trace_once = PTHREAD_ONCE_INIT;

static void trace_make(void) {
    pthread_key_create(&trace_key, NULL);
}

void llr_trace_push(struct llr_frame *slot) {
    pthread_once(&trace_once, trace_make);
    slot->prev = pthread_getspecific(trace_key);
    pthread_setspecific(trace_key, slot);
}

void llr_trace_pop(struct llr_frame *slot) {
    pthread_setspecific(trace_key, slot->prev);
}

static struct llr_frame *trace_head(void) {
    pthread_once(&trace_once, trace_make);
    return pthread_getspecific(trace_key);
}

static int skip_frame(const char *name) {
    return strcmp(name, "sys.abort") == 0 || strcmp(name, "sys.panic") == 0;
}

int32_t llr_call_child(int32_t (*fn)(int64_t), int64_t arg) {
    int pid = fork();
    if (pid == 0) {
        _exit(fn(arg));
    }
    return pid;
}

int32_t llr_spawn_self(const char *arg) {
    (void)arg;
    return -1;
}

/* argv is file plus NUL-separated args of length n. Exit code, 128+signal, or -1. */
int32_t llr_exec(const char *file, const char *args, uint64_t n) {
    int extra = 0;
    uint64_t i;
    int k;
    char **argv;
    pid_t pid;
    int status;
    if (file == 0) {
        return -1;
    }
    if (args != 0 && n > 0) {
        extra = 1;
        for (i = 0; i + 1 < n; i++) {
            if (args[i] == 0) {
                extra++;
            }
        }
    }
    argv = (char **)calloc((size_t)extra + 2, sizeof(char *));
    if (argv == 0) {
        return -1;
    }
    argv[0] = (char *)file;
    k = 1;
    i = 0;
    while (args != 0 && i < n) {
        argv[k++] = (char *)(args + i);
        while (i < n && args[i] != 0) {
            i++;
        }
        if (i < n && args[i] == 0) {
            i++;
        }
    }
    argv[k] = 0;
    pid = fork();
    if (pid < 0) {
        free(argv);
        return -1;
    }
    if (pid == 0) {
        execvp(file, argv);
        _exit(127);
    }
    status = 0;
    if (waitpid(pid, &status, 0) < 0) {
        free(argv);
        return -1;
    }
    free(argv);
    if (WIFEXITED(status)) {
        return WEXITSTATUS(status);
    }
    if (WIFSIGNALED(status)) {
        return 128 + WTERMSIG(status);
    }
    return -1;
}

void llr_panic_frames(void) {
    struct llr_frame *f = trace_head();
    int n = 0;
    while (f != 0 && n < 32) {
        const char *name = f->name;
        if (name != 0 && !skip_frame(name)) {
            unsigned long len = 0;
            while (name[len] != 0) {
                len++;
            }
            write(2, "P|", 2);
            write(2, name, len);
            write(2, "\n", 1);
        }
        f = f->prev;
        n++;
    }
}

#ifdef __APPLE__
#include <errno.h>
#include <fcntl.h>
#include <sys/resource.h>
#include <sys/socket.h>

/* Linux open(2) bits. Callers keep these; Darwin's values are different. */
#define L_CREAT 64
#define L_EXCL 128
#define L_TRUNC 512
#define L_APPEND 1024
#define L_NONBLOCK 2048

static int to_darwin_flags(int flags) {
    int out = flags & 3;
    if (flags & L_CREAT) {
        out |= O_CREAT;
    }
    if (flags & L_EXCL) {
        out |= O_EXCL;
    }
    if (flags & L_TRUNC) {
        out |= O_TRUNC;
    }
    if (flags & L_APPEND) {
        out |= O_APPEND;
    }
    if (flags & L_NONBLOCK) {
        out |= O_NONBLOCK;
    }
    return out;
}

static int to_linux_flags(int flags) {
    int out = flags & 3;
    if (flags & O_CREAT) {
        out |= L_CREAT;
    }
    if (flags & O_EXCL) {
        out |= L_EXCL;
    }
    if (flags & O_TRUNC) {
        out |= L_TRUNC;
    }
    if (flags & O_APPEND) {
        out |= L_APPEND;
    }
    if (flags & O_NONBLOCK) {
        out |= L_NONBLOCK;
    }
    return out;
}

int llr_open(const char *path, int flags, int mode) {
    return open(path, to_darwin_flags(flags), mode);
}

int llr_fcntl(int fd, int cmd, int arg) {
    int fl;
    if (cmd == F_GETFL) {
        fl = fcntl(fd, F_GETFL, 0);
        if (fl < 0) {
            return fl;
        }
        return to_linux_flags(fl);
    }
    if (cmd == F_SETFL) {
        return fcntl(fd, F_SETFL, to_darwin_flags(arg));
    }
    return fcntl(fd, cmd, arg);
}

/* Linux sockaddr_in starts with a 16-bit family. Darwin starts with sin_len. */
static int call_addr(int fd, const void *addr, uint32_t len, int (*call)(int, const struct sockaddr *, socklen_t)) {
    unsigned char buf[16];
    const unsigned char *s = addr;
    if (s != 0 && len == 16 && s[0] == 2 && s[1] == 0) {
        memset(buf, 0, 16);
        buf[0] = 16;
        buf[1] = 2;
        memcpy(buf + 2, s + 2, 14);
        return call(fd, (const struct sockaddr *)buf, len);
    }
    return call(fd, addr, (socklen_t)len);
}

int llr_bind(int fd, const void *addr, uint32_t len) {
    return call_addr(fd, addr, len, bind);
}

int llr_connect(int fd, const void *addr, uint32_t len) {
    return call_addr(fd, addr, len, connect);
}

int llr_socket(int domain, int ty, int proto) {
    int fd = socket(domain, ty, proto);
    int one = 1;
    if (fd < 0) {
        return fd;
    }
    setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &one, sizeof one);
    return fd;
}

int llr_setrlimit(int resource, const void *lim) {
    if (resource == 7) {
        resource = RLIMIT_NOFILE;
    }
    return setrlimit(resource, lim);
}

int *__errno_location(void) {
    static _Thread_local int cached;
    int e = *__error();
    if (e == EAGAIN) {
        e = 11;
    }
    cached = e;
    return &cached;
}
#endif
