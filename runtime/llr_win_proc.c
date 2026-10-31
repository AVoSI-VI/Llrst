/* Process half of the Windows POSIX shim. The fd table lives in llr_win.c. */
#include "llr_win.h"

#define CHILD_MAX 64

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

static Kid kids[CHILD_MAX];

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

typedef struct cmdbuf {
    char *p;
    size_t len;
    size_t cap;
    int bad;
} Cmdbuf;

static void cmd_ch(Cmdbuf *b, char c) {
    char *neu;
    if (b->bad) {
        return;
    }
    if (b->len + 2 >= b->cap) {
        size_t ncap = b->cap == 0 ? 64 : b->cap * 2;
        neu = (char *)realloc(b->p, ncap);
        if (neu == NULL) {
            b->bad = 1;
            return;
        }
        b->p = neu;
        b->cap = ncap;
    }
    b->p[b->len++] = c;
}

static void cmd_bs(Cmdbuf *b, size_t n) {
    while (n > 0) {
        cmd_ch(b, '\\');
        n--;
    }
}

static int cmd_quote(const char *s) {
    const char *p;
    if (s == NULL || s[0] == 0) {
        return 1;
    }
    for (p = s; *p != 0; p++) {
        if (*p == ' ' || *p == '\t' || *p == '"') {
            return 1;
        }
    }
    return 0;
}

/* CommandLineToArgvW quoting, so the child sees the same vector. */
static void cmd_arg(Cmdbuf *b, const char *s) {
    size_t bs;
    if (b->len > 0) {
        cmd_ch(b, ' ');
    }
    if (s == NULL) {
        s = "";
    }
    if (!cmd_quote(s)) {
        while (*s != 0) {
            cmd_ch(b, *s++);
        }
        return;
    }
    cmd_ch(b, '"');
    bs = 0;
    for (; *s != 0; s++) {
        if (*s == '\\') {
            bs++;
        } else if (*s == '"') {
            cmd_bs(b, bs * 2 + 1);
            cmd_ch(b, '"');
            bs = 0;
        } else {
            cmd_bs(b, bs);
            cmd_ch(b, *s);
            bs = 0;
        }
    }
    cmd_bs(b, bs * 2);
    cmd_ch(b, '"');
}

int32_t llr_exec(const char *file, const char *args, uint64_t n) {
    Cmdbuf b;
    uint64_t i;
    STARTUPINFOA start;
    PROCESS_INFORMATION info;
    BOOL ok;
    DWORD code;
    if (file == NULL) {
        return -1;
    }
    memset(&b, 0, sizeof b);
    cmd_arg(&b, file);
    i = 0;
    while (args != NULL && i < n) {
        cmd_arg(&b, args + i);
        while (i < n && args[i] != 0) {
            i++;
        }
        if (i < n && args[i] == 0) {
            i++;
        }
    }
    cmd_ch(&b, 0);
    if (b.bad || b.p == NULL) {
        free(b.p);
        return -1;
    }
    memset(&start, 0, sizeof start);
    start.cb = sizeof start;
    memset(&info, 0, sizeof info);
    ok = CreateProcessA(NULL, b.p, NULL, NULL, TRUE, 0, NULL, NULL, &start, &info);
    free(b.p);
    if (!ok) {
        return -1;
    }
    WaitForSingleObject(info.hProcess, INFINITE);
    code = 1;
    GetExitCodeProcess(info.hProcess, &code);
    CloseHandle(info.hProcess);
    CloseHandle(info.hThread);
    return (int32_t)code;
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
