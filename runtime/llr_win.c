/* Windows stand-in for the POSIX externs in std/sys_windows.llr.
 * File descriptors below 256 are the C runtime. Higher numbers are handles or
 * sockets in this file. /tmp maps at GetTempPath so the existing tests keep
 * their paths. Open flags are the Linux values baked into std/fs.llr. */
#define _CRT_DECLARE_NONSTDC_NAMES 0
#define _CRT_SECURE_NO_WARNINGS
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <errno.h>
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef fileno
#undef fileno
#endif
#ifdef popen
#undef popen
#endif
#ifdef pclose
#undef pclose
#endif
#ifdef isatty
#undef isatty
#endif
#ifdef mkdir
#undef mkdir
#endif
#ifdef getcwd
#undef getcwd
#endif

#define FD_BASE 256
/* Client threads share this table with the server. http_many holds 3072 of each. */
#define FD_MAX 8192
#define KIND_FILE 1
#define KIND_SOCK 2
#define L_CREAT 64
#define L_TRUNC 512
#define L_APPEND 1024
#define L_NONBLOCK 2048
#define F_GETFL 3
#define F_SETFL 4
#define CHILD_MAX 64
#define PATH_CAP 4096

#define SOCK_BAD (~(uint64_t)0)

int llr_wsa_startup(void);
int llr_wsa_last_error(void);
int llr_wsa_close(uint64_t sock);
int llr_wsa_nonblock(uint64_t sock, int on);
uint64_t llr_wsa_socket(int domain, int ty, int proto);
int llr_wsa_bind(uint64_t sock, const void *addr, int len);
int llr_wsa_listen(uint64_t sock, int backlog);
int llr_wsa_connect(uint64_t sock, const void *addr, int len);
int llr_wsa_getsockname(uint64_t sock, void *addr, int *len);
uint64_t llr_wsa_accept(uint64_t sock, void *addr, int *len);
int llr_wsa_send(uint64_t sock, const void *buf, int n, int flags);
int llr_wsa_recv(uint64_t sock, void *buf, int n, int flags);
int llr_wsa_poll(void *fds, uint32_t n, int timeout);
short llr_wsa_to_events(short events);
short llr_wsa_from_events(short events);

typedef struct slot {
    int kind;
    HANDLE handle;
    uint64_t sock;
    int nonblock;
    int listening;
} Slot;

typedef struct job {
    int32_t (*fn)(int64_t);
    int64_t arg;
    volatile int32_t code;
} Job;

typedef struct kid {
    int live;
    HANDLE thread;
    Job *job;
} Kid;

static Slot slots[FD_MAX];
static Kid kids[CHILD_MAX];
static CRITICAL_SECTION fd_lock;
static INIT_ONCE once = INIT_ONCE_STATIC_INIT;

static BOOL CALLBACK init_cb(PINIT_ONCE o, PVOID p, PVOID *ctx) {
    (void)o;
    (void)p;
    (void)ctx;
    InitializeCriticalSection(&fd_lock);
    _setmode(0, _O_BINARY);
    _setmode(1, _O_BINARY);
    _setmode(2, _O_BINARY);
    llr_wsa_startup();
    return TRUE;
}

static void ensure_init(void) {
    InitOnceExecuteOnce(&once, init_cb, NULL, NULL);
}

static void set_errno(int err) {
    errno = err;
}

int *__errno_location(void) {
    ensure_init();
    return _errno();
}

static void map_wsa(void) {
    int err = llr_wsa_last_error();
    if (err == 10035) {
        set_errno(11);
    } else if (err == 10004) {
        set_errno(4);
    } else if (err > 0 && err < 256) {
        set_errno(err);
    } else {
        set_errno(5);
    }
}

static int map_path(const char *in, char *out, size_t cap) {
    if (in[0] == '/' && in[1] == 't' && in[2] == 'm' && in[3] == 'p' && (in[4] == 0 || in[4] == '/')) {
        char temp[MAX_PATH];
        DWORD n = GetTempPathA(MAX_PATH, temp);
        if (n == 0 || n >= MAX_PATH) {
            return -1;
        }
        if (in[4] == 0) {
            if (n > 0 && (temp[n - 1] == '\\' || temp[n - 1] == '/')) {
                temp[n - 1] = 0;
            }
            return snprintf(out, cap, "%s", temp) < (int)cap ? 0 : -1;
        }
        return snprintf(out, cap, "%s%s", temp, in + 5) < (int)cap ? 0 : -1;
    }
    return snprintf(out, cap, "%s", in) < (int)cap ? 0 : -1;
}

static int take_new(int kind, HANDLE handle, uint64_t sock) {
    int id = -1;
    EnterCriticalSection(&fd_lock);
    for (int i = 0; i < FD_MAX; i++) {
        if (slots[i].kind == 0) {
            slots[i].kind = kind;
            slots[i].handle = handle;
            slots[i].sock = sock;
            slots[i].nonblock = 0;
            slots[i].listening = 0;
            id = FD_BASE + i;
            break;
        }
    }
    LeaveCriticalSection(&fd_lock);
    return id;
}

static int copy_slot(int fd, Slot *out) {
    if (fd < FD_BASE || fd >= FD_BASE + FD_MAX) {
        return 0;
    }
    EnterCriticalSection(&fd_lock);
    Slot *slot = &slots[fd - FD_BASE];
    int ok = slot->kind != 0;
    if (ok) {
        *out = *slot;
    }
    LeaveCriticalSection(&fd_lock);
    return ok;
}

static int take_slot(int fd, Slot *out) {
    if (fd < FD_BASE || fd >= FD_BASE + FD_MAX) {
        return 0;
    }
    EnterCriticalSection(&fd_lock);
    Slot *slot = &slots[fd - FD_BASE];
    int ok = slot->kind != 0;
    if (ok) {
        *out = *slot;
        slot->kind = 0;
    }
    LeaveCriticalSection(&fd_lock);
    return ok;
}

static HANDLE std_handle(int fd) {
    if (fd == 0) {
        return GetStdHandle(STD_INPUT_HANDLE);
    }
    if (fd == 1) {
        return GetStdHandle(STD_OUTPUT_HANDLE);
    }
    return GetStdHandle(STD_ERROR_HANDLE);
}

int64_t write(int fd, const void *buf, uint64_t n) {
    ensure_init();
    if (n > 0x7fffffff) {
        n = 0x7fffffff;
    }
    if (fd >= 0 && fd <= 2) {
        DWORD wrote = 0;
        if (!WriteFile(std_handle(fd), buf, (DWORD)n, &wrote, NULL)) {
            set_errno(5);
            return -1;
        }
        return (int64_t)wrote;
    }
    if (fd < FD_BASE) {
        return _write(fd, buf, (unsigned)n);
    }
    Slot slot;
    if (!copy_slot(fd, &slot) || slot.kind != KIND_FILE) {
        set_errno(9);
        return -1;
    }
    DWORD wrote = 0;
    if (!WriteFile(slot.handle, buf, (DWORD)n, &wrote, NULL)) {
        set_errno(5);
        return -1;
    }
    return (int64_t)wrote;
}

int64_t read(int fd, void *buf, uint64_t n) {
    ensure_init();
    if (n > 0x7fffffff) {
        n = 0x7fffffff;
    }
    if (fd >= 0 && fd <= 2) {
        DWORD got = 0;
        if (!ReadFile(std_handle(fd), buf, (DWORD)n, &got, NULL)) {
            set_errno(5);
            return -1;
        }
        return (int64_t)got;
    }
    if (fd < FD_BASE) {
        return _read(fd, buf, (unsigned)n);
    }
    Slot slot;
    if (!copy_slot(fd, &slot) || slot.kind != KIND_FILE) {
        set_errno(9);
        return -1;
    }
    DWORD got = 0;
    if (!ReadFile(slot.handle, buf, (DWORD)n, &got, NULL)) {
        set_errno(5);
        return -1;
    }
    return (int64_t)got;
}

int open(const char *path, int flags, int mode) {
    (void)mode;
    ensure_init();
    char mapped[PATH_CAP];
    if (map_path(path, mapped, sizeof mapped) != 0) {
        set_errno(2);
        return -1;
    }
    int access_bits = flags & 3;
    DWORD access = GENERIC_READ;
    if (access_bits == 1) {
        access = GENERIC_WRITE;
    } else if (access_bits == 2) {
        access = GENERIC_READ | GENERIC_WRITE;
    }
    if (flags & L_APPEND) {
        access = FILE_APPEND_DATA;
    }
    DWORD disp = OPEN_EXISTING;
    if ((flags & L_CREAT) && (flags & L_TRUNC)) {
        disp = CREATE_ALWAYS;
    } else if (flags & L_CREAT) {
        disp = OPEN_ALWAYS;
    } else if (flags & L_TRUNC) {
        disp = TRUNCATE_EXISTING;
    }
    HANDLE handle = CreateFileA(mapped, access, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, disp, FILE_ATTRIBUTE_NORMAL, NULL);
    if (handle == INVALID_HANDLE_VALUE) {
        set_errno(2);
        return -1;
    }
    int fd = take_new(KIND_FILE, handle, SOCK_BAD);
    if (fd < 0) {
        CloseHandle(handle);
        set_errno(24);
        return -1;
    }
    return fd;
}

int close(int fd) {
    ensure_init();
    if (fd >= 0 && fd <= 2) {
        return 0;
    }
    if (fd < FD_BASE) {
        return _close(fd);
    }
    Slot slot;
    if (!take_slot(fd, &slot)) {
        set_errno(9);
        return -1;
    }
    if (slot.kind == KIND_SOCK) {
        llr_wsa_close(slot.sock);
    } else {
        CloseHandle(slot.handle);
    }
    return 0;
}

int unlink(const char *path) {
    char mapped[PATH_CAP];
    if (map_path(path, mapped, sizeof mapped) != 0) {
        set_errno(2);
        return -1;
    }
    if (DeleteFileA(mapped)) {
        return 0;
    }
    if (GetLastError() != ERROR_FILE_NOT_FOUND && GetLastError() != ERROR_PATH_NOT_FOUND) {
        set_errno(13);
        return -1;
    }
    char exe[PATH_CAP + 8];
    snprintf(exe, sizeof exe, "%s.exe", mapped);
    if (DeleteFileA(exe)) {
        return 0;
    }
    set_errno(2);
    return -1;
}

int mkdir(const char *path, int mode) {
    (void)mode;
    char mapped[PATH_CAP];
    if (map_path(path, mapped, sizeof mapped) != 0) {
        set_errno(2);
        return -1;
    }
    if (CreateDirectoryA(mapped, NULL)) {
        return 0;
    }
    set_errno(GetLastError() == ERROR_ALREADY_EXISTS ? 17 : 2);
    return -1;
}

int rename(const char *from, const char *to) {
    char a[PATH_CAP];
    char b[PATH_CAP];
    if (map_path(from, a, sizeof a) != 0 || map_path(to, b, sizeof b) != 0) {
        set_errno(2);
        return -1;
    }
    if (MoveFileExA(a, b, MOVEFILE_REPLACE_EXISTING)) {
        return 0;
    }
    set_errno(2);
    return -1;
}

char *getcwd(char *buf, uint64_t n) {
    if (buf == NULL || n == 0 || n > 0xffffffffu) {
        set_errno(22);
        return NULL;
    }
    DWORD got = GetCurrentDirectoryA((DWORD)n, buf);
    if (got == 0 || got >= n) {
        set_errno(34);
        return NULL;
    }
    return buf;
}

typedef struct dir {
    HANDLE handle;
    WIN32_FIND_DATAA data;
    int first;
} Dir;

void *opendir(const char *path) {
    char mapped[PATH_CAP];
    if (map_path(path, mapped, sizeof mapped) != 0) {
        return NULL;
    }
    size_t n = strlen(mapped);
    int slash = n > 0 && (mapped[n - 1] == '/' || mapped[n - 1] == '\\');
    char pat[PATH_CAP + 4];
    if (snprintf(pat, sizeof pat, slash ? "%s*" : "%s\\*", mapped) >= (int)sizeof pat) {
        return NULL;
    }
    Dir *dir = (Dir *)calloc(1, sizeof(Dir));
    if (dir == NULL) {
        return NULL;
    }
    dir->handle = FindFirstFileA(pat, &dir->data);
    if (dir->handle == INVALID_HANDLE_VALUE) {
        free(dir);
        return NULL;
    }
    dir->first = 1;
    return dir;
}

void *readdir(void *dirp) {
    Dir *dir = (Dir *)dirp;
    if (dir == NULL) {
        return NULL;
    }
    if (dir->first) {
        dir->first = 0;
        return &dir->data;
    }
    if (!FindNextFileA(dir->handle, &dir->data)) {
        return NULL;
    }
    return &dir->data;
}

int closedir(void *dirp) {
    Dir *dir = (Dir *)dirp;
    if (dir == NULL) {
        return -1;
    }
    if (dir->handle != INVALID_HANDLE_VALUE) {
        FindClose(dir->handle);
    }
    free(dir);
    return 0;
}

const char *llr_readdir_name(const void *ent) {
    if (ent == NULL) {
        return NULL;
    }
    return ((const WIN32_FIND_DATAA *)ent)->cFileName;
}

static int file_info(const char *path, WIN32_FILE_ATTRIBUTE_DATA *info) {
    char mapped[PATH_CAP];
    if (map_path(path, mapped, sizeof mapped) != 0) {
        return -1;
    }
    if (!GetFileAttributesExA(mapped, GetFileExInfoStandard, info)) {
        return -1;
    }
    return 0;
}

int64_t llr_stat_mtime(const char *path, int64_t *out) {
    WIN32_FILE_ATTRIBUTE_DATA info;
    if (file_info(path, &info) != 0) {
        return -1;
    }
    uint64_t ticks = ((uint64_t)info.ftLastWriteTime.dwHighDateTime << 32) | info.ftLastWriteTime.dwLowDateTime;
    const uint64_t epoch = 116444736000000000ULL;
    *out = ticks < epoch ? 0 : (int64_t)((ticks - epoch) / 10000000ULL);
    return 0;
}

int64_t llr_stat_size(const char *path, int64_t *out) {
    WIN32_FILE_ATTRIBUTE_DATA info;
    if (file_info(path, &info) != 0) {
        return -1;
    }
    *out = (int64_t)(((uint64_t)info.nFileSizeHigh << 32) | info.nFileSizeLow);
    return 0;
}

struct llr_frame {
    struct llr_frame *prev;
    const char *name;
};

extern struct llr_frame *llr_trace;

static int skip_frame(const char *name) {
    return strcmp(name, "sys.abort") == 0 || strcmp(name, "sys.panic") == 0;
}

void llr_panic_frames(void) {
    struct llr_frame *frame = llr_trace;
    int n = 0;
    while (frame != NULL && n < 32) {
        const char *name = frame->name;
        if (name != NULL && !skip_frame(name)) {
            unsigned long len = 0;
            while (name[len] != 0) {
                len++;
            }
            write(2, "P|", 2);
            write(2, name, len);
            write(2, "\n", 1);
        }
        frame = frame->prev;
        n++;
    }
}

static int sock_fd(uint64_t sock) {
    int fd = take_new(KIND_SOCK, NULL, sock);
    if (fd < 0) {
        llr_wsa_close(sock);
        set_errno(24);
    }
    return fd;
}

static uint64_t socket_of(int fd) {
    Slot slot;
    if (!copy_slot(fd, &slot) || slot.kind != KIND_SOCK) {
        set_errno(9);
        return SOCK_BAD;
    }
    return slot.sock;
}

int socket(int domain, int ty, int proto) {
    ensure_init();
    uint64_t sock = llr_wsa_socket(domain, ty, proto);
    if (sock == SOCK_BAD) {
        map_wsa();
        return -1;
    }
    return sock_fd(sock);
}

int bind(int fd, void *addr, uint32_t len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (llr_wsa_bind(sock, addr, (int)len) != 0) {
        map_wsa();
        return -1;
    }
    return 0;
}

int listen(int fd, int backlog) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (llr_wsa_listen(sock, backlog) != 0) {
        int again = 0;
        EnterCriticalSection(&fd_lock);
        if (fd >= FD_BASE && fd < FD_BASE + FD_MAX && slots[fd - FD_BASE].listening) {
            again = 1;
        }
        LeaveCriticalSection(&fd_lock);
        if (again) {
            return 0;
        }
        map_wsa();
        return -1;
    }
    EnterCriticalSection(&fd_lock);
    if (fd >= FD_BASE && fd < FD_BASE + FD_MAX) {
        slots[fd - FD_BASE].listening = 1;
    }
    LeaveCriticalSection(&fd_lock);
    return 0;
}

int accept(int fd, void *addr, uint32_t *len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    int got = len == NULL ? 0 : (int)*len;
    uint64_t client = llr_wsa_accept(sock, addr, len == NULL ? NULL : &got);
    if (client == SOCK_BAD) {
        map_wsa();
        return -1;
    }
    if (len != NULL) {
        *len = (uint32_t)got;
    }
    return sock_fd(client);
}

int connect(int fd, void *addr, uint32_t len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (llr_wsa_connect(sock, addr, (int)len) != 0) {
        map_wsa();
        return -1;
    }
    return 0;
}

int getsockname(int fd, void *addr, uint32_t *len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    int got = len == NULL ? 0 : (int)*len;
    if (llr_wsa_getsockname(sock, addr, &got) != 0) {
        map_wsa();
        return -1;
    }
    if (len != NULL) {
        *len = (uint32_t)got;
    }
    return 0;
}

int64_t send(int fd, const void *buf, uint64_t n, int flags) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (n > 0x7fffffff) {
        n = 0x7fffffff;
    }
    int sent = llr_wsa_send(sock, buf, (int)n, flags);
    if (sent < 0) {
        map_wsa();
        return -1;
    }
    return sent;
}

int64_t recv(int fd, void *buf, uint64_t n, int flags) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (n > 0x7fffffff) {
        n = 0x7fffffff;
    }
    int got = llr_wsa_recv(sock, buf, (int)n, flags);
    if (got < 0) {
        map_wsa();
        return -1;
    }
    return got;
}

struct lpoll {
    int32_t fd;
    int16_t events;
    int16_t revents;
};

struct wsa_poll {
    uint64_t sock;
    int16_t events;
    int16_t revents;
    int16_t pad1;
    int16_t pad2;
};

int poll(void *base, uint64_t n, int timeout) {
    ensure_init();
    if (n > 4096) {
        n = 4096;
    }
    struct lpoll *in = (struct lpoll *)base;
    struct wsa_poll *wsa = (struct wsa_poll *)calloc((size_t)n, sizeof(struct wsa_poll));
    if (n > 0 && wsa == NULL) {
        set_errno(12);
        return -1;
    }
    for (uint64_t i = 0; i < n; i++) {
        Slot slot;
        wsa[i].sock = copy_slot(in[i].fd, &slot) && slot.kind == KIND_SOCK ? slot.sock : SOCK_BAD;
        wsa[i].events = llr_wsa_to_events(in[i].events);
        wsa[i].revents = 0;
    }
    int ready = llr_wsa_poll(wsa, (uint32_t)n, timeout);
    if (ready < 0) {
        map_wsa();
        free(wsa);
        return -1;
    }
    for (uint64_t i = 0; i < n; i++) {
        in[i].revents = llr_wsa_from_events(wsa[i].revents);
    }
    free(wsa);
    return ready;
}

int fcntl(int fd, int cmd, int arg) {
    ensure_init();
    if (fd < FD_BASE || fd >= FD_BASE + FD_MAX) {
        set_errno(9);
        return -1;
    }
    EnterCriticalSection(&fd_lock);
    Slot *slot = &slots[fd - FD_BASE];
    if (slot->kind != KIND_SOCK) {
        LeaveCriticalSection(&fd_lock);
        set_errno(9);
        return -1;
    }
    if (cmd == F_GETFL) {
        int flags = slot->nonblock ? L_NONBLOCK : 0;
        LeaveCriticalSection(&fd_lock);
        return flags;
    }
    if (cmd != F_SETFL) {
        LeaveCriticalSection(&fd_lock);
        set_errno(22);
        return -1;
    }
    uint64_t sock = slot->sock;
    LeaveCriticalSection(&fd_lock);
    if (llr_wsa_nonblock(sock, (arg & L_NONBLOCK) ? 1 : 0) != 0) {
        map_wsa();
        return -1;
    }
    EnterCriticalSection(&fd_lock);
    slots[fd - FD_BASE].nonblock = (arg & L_NONBLOCK) ? 1 : 0;
    LeaveCriticalSection(&fd_lock);
    return 0;
}


int fork(void) {
    set_errno(1);
    return -1;
}

static DWORD WINAPI child_main(LPVOID param) {
    Job *job = (Job *)param;
    job->code = job->fn(job->arg);
    return 0;
}

int32_t llr_call_child(int32_t (*fn)(int64_t), int64_t arg) {
    ensure_init();
    int id = -1;
    for (int i = 1; i < CHILD_MAX; i++) {
        if (!kids[i].live) {
            id = i;
            break;
        }
    }
    if (id < 0) {
        return -1;
    }
    Job *job = (Job *)calloc(1, sizeof(Job));
    if (job == NULL) {
        return -1;
    }
    job->fn = fn;
    job->arg = arg;
    job->code = 1;
    HANDLE thread = CreateThread(NULL, 4u * 1024u * 1024u, child_main, job, 0, NULL);
    if (thread == NULL) {
        free(job);
        return -1;
    }
    kids[id].live = 1;
    kids[id].thread = thread;
    kids[id].job = job;
    return id;
}

int waitpid(int pid, int *status, int opts) {
    (void)opts;
    if (pid < 1 || pid >= CHILD_MAX || !kids[pid].live) {
        set_errno(10);
        return -1;
    }
    WaitForSingleObject(kids[pid].thread, INFINITE);
    CloseHandle(kids[pid].thread);
    int code = kids[pid].job->code;
    free(kids[pid].job);
    kids[pid].live = 0;
    if (status != NULL) {
        *status = (code & 255) << 8;
    }
    return pid;
}

typedef void *(*llr_start_fn)(void *);

struct start_pack {
    llr_start_fn fn;
    void *arg;
};

static DWORD WINAPI start_main(LPVOID param) {
    struct start_pack *pack = (struct start_pack *)param;
    llr_start_fn fn = pack->fn;
    void *arg = pack->arg;
    free(pack);
    fn(arg);
    return 0;
}

int pthread_create(void *thread, void *attr, llr_start_fn start, void *arg) {
    (void)attr;
    struct start_pack *pack = (struct start_pack *)malloc(sizeof(*pack));
    if (pack == NULL) {
        return 11;
    }
    pack->fn = start;
    pack->arg = arg;
    HANDLE handle = CreateThread(NULL, 4u * 1024u * 1024u, start_main, pack, 0, NULL);
    if (handle == NULL) {
        free(pack);
        return 1;
    }
    uint64_t id = (uint64_t)(uintptr_t)handle;
    memcpy(thread, &id, sizeof id);
    return 0;
}

int pthread_join(uint64_t thread, void *ret) {
    (void)ret;
    HANDLE handle = (HANDLE)(uintptr_t)thread;
    WaitForSingleObject(handle, INFINITE);
    CloseHandle(handle);
    return 0;
}

int pthread_mutex_init(void *mutex, void *attr) {
    (void)attr;
    InitializeCriticalSection((CRITICAL_SECTION *)mutex);
    return 0;
}

int pthread_mutex_lock(void *mutex) {
    EnterCriticalSection((CRITICAL_SECTION *)mutex);
    return 0;
}

int pthread_mutex_unlock(void *mutex) {
    LeaveCriticalSection((CRITICAL_SECTION *)mutex);
    return 0;
}

int pthread_mutex_destroy(void *mutex) {
    DeleteCriticalSection((CRITICAL_SECTION *)mutex);
    return 0;
}

int pthread_cond_init(void *cond, void *attr) {
    (void)attr;
    InitializeConditionVariable((CONDITION_VARIABLE *)cond);
    return 0;
}

int pthread_cond_wait(void *cond, void *mutex) {
    SleepConditionVariableCS((CONDITION_VARIABLE *)cond, (CRITICAL_SECTION *)mutex, INFINITE);
    return 0;
}

int pthread_cond_broadcast(void *cond) {
    WakeAllConditionVariable((CONDITION_VARIABLE *)cond);
    return 0;
}

int pthread_cond_destroy(void *cond) {
    (void)cond;
    return 0;
}

int setrlimit(int resource, void *lim) {
    (void)resource;
    (void)lim;
    return 0;
}

static int shift_status(int code) {
    if (code < 0) {
        return code;
    }
    return (code & 255) << 8;
}

int system(const char *cmd) {
    ensure_init();
    size_t cap = strlen(cmd) + 16;
    char *line = (char *)malloc(cap);
    if (line == NULL) {
        return -1;
    }
    snprintf(line, cap, "cmd.exe /c %s", cmd);
    STARTUPINFOA start;
    PROCESS_INFORMATION info;
    memset(&start, 0, sizeof start);
    start.cb = sizeof start;
    memset(&info, 0, sizeof info);
    BOOL ok = CreateProcessA(NULL, line, NULL, NULL, TRUE, 0, NULL, NULL, &start, &info);
    free(line);
    if (!ok) {
        return -1;
    }
    WaitForSingleObject(info.hProcess, INFINITE);
    DWORD code = 1;
    GetExitCodeProcess(info.hProcess, &code);
    CloseHandle(info.hProcess);
    CloseHandle(info.hThread);
    return shift_status((int)code);
}

FILE *popen(const char *cmd, const char *mode) {
    ensure_init();
    FILE *file = _popen(cmd, mode);
    if (file != NULL) {
        _setmode(_fileno(file), _O_BINARY);
    }
    return file;
}

int fileno(FILE *file) {
    return _fileno(file);
}

int pclose(FILE *file) {
    return shift_status(_pclose(file));
}

int isatty(int fd) {
    ensure_init();
    if (fd >= 0 && fd <= 2) {
        DWORD mode = 0;
        return GetConsoleMode(std_handle(fd), &mode) ? 1 : 0;
    }
    if (fd >= FD_BASE) {
        return 0;
    }
    return _isatty(fd);
}

static char *child_env(const char *arg) {
    char *old = GetEnvironmentStringsA();
    if (old == NULL) {
        return NULL;
    }
    size_t n = 0;
    while (!(old[n] == 0 && old[n + 1] == 0)) {
        n++;
    }
    n++;
    const char *key = "LLR_TEST_CHILD=";
    size_t add = strlen(key) + strlen(arg) + 1;
    char *neu = (char *)malloc(n + add + 1);
    if (neu == NULL) {
        FreeEnvironmentStringsA(old);
        return NULL;
    }
    memcpy(neu, old, n);
    memcpy(neu + n, key, strlen(key));
    memcpy(neu + n + strlen(key), arg, strlen(arg) + 1);
    neu[n + add] = 0;
    FreeEnvironmentStringsA(old);
    return neu;
}

int32_t llr_spawn_self(const char *arg) {
    char exe[1024];
    if (GetModuleFileNameA(NULL, exe, sizeof exe) == 0) {
        return -1;
    }
    size_t cap = strlen(exe) + strlen(arg) + 8;
    char *line = (char *)malloc(cap);
    char *env = child_env(arg);
    if (line == NULL || env == NULL) {
        free(line);
        free(env);
        return -1;
    }
    snprintf(line, cap, "\"%s\" \"%s\"", exe, arg);
    STARTUPINFOA start;
    PROCESS_INFORMATION info;
    memset(&start, 0, sizeof start);
    start.cb = sizeof start;
    memset(&info, 0, sizeof info);
    BOOL ok = CreateProcessA(exe, line, NULL, NULL, TRUE, 0, env, NULL, &start, &info);
    free(line);
    free(env);
    if (!ok) {
        return -1;
    }
    WaitForSingleObject(info.hProcess, INFINITE);
    DWORD code = 1;
    GetExitCodeProcess(info.hProcess, &code);
    CloseHandle(info.hProcess);
    CloseHandle(info.hThread);
    return (int32_t)code;
}

/* Bytes written, not including the NUL, or -1 when the buffer is too small. */
int64_t llr_exe_path(char *buf, uint64_t n) {
    DWORD cap;
    DWORD got;
    if (buf == 0 || n < 2 || n > 0xffffffffu) {
        return -1;
    }
    cap = (DWORD)n;
    got = GetModuleFileNameA(NULL, buf, cap);
    if (got == 0 || got >= cap) {
        return -1;
    }
    return (int64_t)got;
}
