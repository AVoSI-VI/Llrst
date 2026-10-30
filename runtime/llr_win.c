/* Windows stand-in for the POSIX externs in std/sys_windows.llr.
 * File descriptors below 256 are the C runtime. Higher numbers are handles or
 * sockets in this file. /tmp maps at GetTempPath so the existing tests keep
 * their paths. Open flags are the Linux values baked into std/fs.llr. */

#include "llr_win.h"

Slot slots[FD_MAX];
CRITICAL_SECTION fd_lock;
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

void ensure_init(void) {
    InitOnceExecuteOnce(&once, init_cb, NULL, NULL);
}

void set_errno(int err) {
    errno = err;
}

int *__errno_location(void) {
    ensure_init();
    return _errno();
}

void map_wsa(void) {
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

int take_new(int kind, HANDLE handle, uint64_t sock) {
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

int copy_slot(int fd, Slot *out) {
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

int take_slot(int fd, Slot *out) {
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

HANDLE std_handle(int fd) {
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

int rmdir(const char *path) {
    char mapped[PATH_CAP];
    if (map_path(path, mapped, sizeof mapped) != 0) {
        set_errno(2);
        return -1;
    }
    if (RemoveDirectoryA(mapped)) {
        return 0;
    }
    set_errno(2);
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

static DWORD trace_tls = TLS_OUT_OF_INDEXES;
static INIT_ONCE trace_once = INIT_ONCE_STATIC_INIT;

static BOOL CALLBACK trace_init(PINIT_ONCE o, PVOID p, PVOID *ctx) {
    (void)o;
    (void)p;
    (void)ctx;
    trace_tls = TlsAlloc();
    return TRUE;
}

static void trace_ready(void) {
    InitOnceExecuteOnce(&trace_once, trace_init, NULL, NULL);
}

void llr_trace_push(struct llr_frame *slot) {
    trace_ready();
    slot->prev = (struct llr_frame *)TlsGetValue(trace_tls);
    TlsSetValue(trace_tls, slot);
}

void llr_trace_pop(struct llr_frame *slot) {
    trace_ready();
    TlsSetValue(trace_tls, slot->prev);
}

static struct llr_frame *trace_head(void) {
    trace_ready();
    return (struct llr_frame *)TlsGetValue(trace_tls);
}

static int skip_frame(const char *name) {
    return strcmp(name, "sys.abort") == 0 || strcmp(name, "sys.panic") == 0;
}

void llr_panic_frames(void) {
    struct llr_frame *frame = trace_head();
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
