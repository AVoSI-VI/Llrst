/* Wall and monotonic clocks, sleep, entropy, env, pid, and the io line slot.
 * On Windows, llr_run lives here and borrows llr_win_cmdline from llr_win_proc.c. */
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static uint8_t *io_slot;

static volatile int64_t llr_sink;

int64_t llr_keep(int64_t n) {
    llr_sink += n;
    return llr_sink;
}

uint8_t *llr_io_get(void) { return io_slot; }

void llr_io_set(uint8_t *p) { io_slot = p; }

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

extern char *llr_win_cmdline(const char *file, const char *args, uint64_t n);

int64_t llr_now_ms(void) {
    FILETIME ft;
    ULARGE_INTEGER u;
    GetSystemTimeAsFileTime(&ft);
    u.LowPart = ft.dwLowDateTime;
    u.HighPart = ft.dwHighDateTime;
    return (int64_t)(u.QuadPart / 10000ULL) - 11644473600000LL;
}

int64_t llr_mono_ms(void) { return (int64_t)GetTickCount64(); }

int64_t llr_mono_ns(void) {
    LARGE_INTEGER c, f;
    uint64_t q, freq, sec, rem;
    if (!QueryPerformanceCounter(&c) || !QueryPerformanceFrequency(&f) || f.QuadPart <= 0) {
        return 0;
    }
    q = (uint64_t)c.QuadPart;
    freq = (uint64_t)f.QuadPart;
    sec = q / freq;
    rem = q % freq;
    return (int64_t)(sec * 1000000000ULL + rem * 1000000000ULL / freq);
}

void llr_sleep_ms(int64_t ms) {
    if (ms > 0) {
        Sleep((DWORD)ms);
    }
}

int64_t llr_entropy(uint8_t *buf, uint64_t n) {
    HMODULE lib;
    BOOLEAN(WINAPI * fn)(PVOID, ULONG);
    if (buf == 0 || n > 0xffffffffu) {
        return -1;
    }
    lib = LoadLibraryA("advapi32.dll");
    if (lib == 0) {
        return -1;
    }
    fn = (BOOLEAN(WINAPI *)(PVOID, ULONG))GetProcAddress(lib, "SystemFunction036");
    if (fn == 0 || !fn(buf, (ULONG)n)) {
        FreeLibrary(lib);
        return -1;
    }
    FreeLibrary(lib);
    return (int64_t)n;
}

int32_t llr_set_env(const char *name, const char *value) {
    if (name == 0 || value == 0) {
        return -1;
    }
    return _putenv_s(name, value) == 0 ? 0 : -1;
}

int64_t llr_pid(void) { return (int64_t)GetCurrentProcessId(); }

static HANDLE child_end(HANDLE *mine, int parent_reads) {
    SECURITY_ATTRIBUTES sa;
    HANDLE rd = 0;
    HANDLE wr = 0;
    memset(&sa, 0, sizeof sa);
    sa.nLength = sizeof sa;
    sa.bInheritHandle = TRUE;
    if (!CreatePipe(&rd, &wr, &sa, 0)) {
        return 0;
    }
    if (parent_reads) {
        SetHandleInformation(rd, HANDLE_FLAG_INHERIT, 0);
        *mine = rd;
        return wr;
    }
    SetHandleInformation(wr, HANDLE_FLAG_INHERIT, 0);
    *mine = wr;
    return rd;
}

static void close_h(HANDLE h) {
    if (h != 0) {
        CloseHandle(h);
    }
}

static int grab(HANDLE h, uint8_t **out, uint64_t *n) {
    uint64_t cap = 256;
    uint64_t len = 0;
    uint8_t *buf = (uint8_t *)malloc(256);
    DWORD got = 0;
    if (buf == 0) {
        return -1;
    }
    while (ReadFile(h, buf + len, (DWORD)(cap - len), &got, 0) && got > 0) {
        uint8_t *neu;
        len += got;
        if (cap - len >= 256) {
            continue;
        }
        neu = (uint8_t *)realloc(buf, (size_t)(cap * 2));
        if (neu == 0) {
            free(buf);
            return -1;
        }
        buf = neu;
        cap *= 2;
    }
    *out = buf;
    *n = len;
    return 0;
}

int32_t llr_run(const char *file, const char *args, uint64_t n, const char *in, uint64_t in_n,
    uint8_t **out, uint64_t *out_n, uint8_t **err, uint64_t *err_n) {
    char *line;
    HANDLE in_w = 0;
    HANDLE out_r = 0;
    HANDLE err_r = 0;
    HANDLE in_r;
    HANDLE out_w;
    HANDLE err_w;
    STARTUPINFOA start;
    PROCESS_INFORMATION info;
    DWORD code = 1;
    DWORD wrote = 0;
    uint64_t off = 0;
    DWORD last;
    if (out != 0) {
        *out = 0;
    }
    if (out_n != 0) {
        *out_n = 0;
    }
    if (err != 0) {
        *err = 0;
    }
    if (err_n != 0) {
        *err_n = 0;
    }
    if (file == 0 || out == 0 || out_n == 0 || err == 0 || err_n == 0) {
        return -1;
    }
    line = llr_win_cmdline(file, args, n);
    if (line == 0) {
        return -1;
    }
    in_r = child_end(&in_w, 0);
    out_w = child_end(&out_r, 1);
    err_w = child_end(&err_r, 1);
    if (in_r == 0 || out_w == 0 || err_w == 0) {
        close_h(in_r);
        close_h(in_w);
        close_h(out_r);
        close_h(out_w);
        close_h(err_r);
        close_h(err_w);
        free(line);
        return -1;
    }
    memset(&start, 0, sizeof start);
    start.cb = sizeof start;
    start.dwFlags = STARTF_USESTDHANDLES;
    start.hStdInput = in_r;
    start.hStdOutput = out_w;
    start.hStdError = err_w;
    memset(&info, 0, sizeof info);
    if (!CreateProcessA(0, line, 0, 0, TRUE, 0, 0, 0, &start, &info)) {
        last = GetLastError();
        close_h(in_r);
        close_h(in_w);
        close_h(out_r);
        close_h(out_w);
        close_h(err_r);
        close_h(err_w);
        free(line);
        if (last == ERROR_FILE_NOT_FOUND || last == ERROR_PATH_NOT_FOUND) {
            return 127;
        }
        return -1;
    }
    free(line);
    close_h(in_r);
    close_h(out_w);
    close_h(err_w);
    if (in != 0) {
        while (off < in_n) {
            DWORD chunk = (DWORD)(in_n - off > 1048576 ? 1048576 : in_n - off);
            if (!WriteFile(in_w, in + off, chunk, &wrote, 0)) {
                break;
            }
            off += wrote;
        }
    }
    close_h(in_w);
    if (grab(out_r, out, out_n) != 0 || grab(err_r, err, err_n) != 0) {
        close_h(out_r);
        close_h(err_r);
        WaitForSingleObject(info.hProcess, INFINITE);
        CloseHandle(info.hProcess);
        CloseHandle(info.hThread);
        return -1;
    }
    close_h(out_r);
    close_h(err_r);
    WaitForSingleObject(info.hProcess, INFINITE);
    GetExitCodeProcess(info.hProcess, &code);
    CloseHandle(info.hProcess);
    CloseHandle(info.hThread);
    return (int32_t)code;
}

#else
#include <errno.h>
#include <fcntl.h>
#include <time.h>
#include <unistd.h>

int64_t llr_now_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
}

int64_t llr_mono_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
}

int64_t llr_mono_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000000000 + (int64_t)ts.tv_nsec;
}

void llr_sleep_ms(int64_t ms) {
    struct timespec ts;
    if (ms <= 0) {
        return;
    }
    ts.tv_sec = (time_t)(ms / 1000);
    ts.tv_nsec = (long)((ms % 1000) * 1000000L);
    while (nanosleep(&ts, &ts) != 0 && errno == EINTR) {
    }
}

int64_t llr_entropy(uint8_t *buf, uint64_t n) {
    int fd;
    uint64_t got = 0;
    if (buf == 0) {
        return -1;
    }
    fd = open("/dev/urandom", O_RDONLY);
    if (fd < 0) {
        return -1;
    }
    while (got < n) {
        ssize_t k = read(fd, buf + got, (size_t)(n - got));
        if (k < 0 && errno == EINTR) {
            continue;
        }
        if (k <= 0) {
            close(fd);
            return -1;
        }
        got += (uint64_t)k;
    }
    close(fd);
    return (int64_t)n;
}

int32_t llr_set_env(const char *name, const char *value) {
    if (name == 0 || value == 0) {
        return -1;
    }
    return setenv(name, value, 1) == 0 ? 0 : -1;
}

int64_t llr_pid(void) { return (int64_t)getpid(); }
#endif
