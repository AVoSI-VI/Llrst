/* Freestanding aarch64 Linux entry for tests/a64.sh. qemu-user runs the binary. */
typedef long long i64;
typedef unsigned long long u64;

__attribute__((noinline)) static i64 svc(i64 n, i64 a, i64 b, i64 c, i64 d, i64 e, i64 f) {
    register i64 x8 __asm__("x8") = n;
    register i64 x0 __asm__("x0") = a;
    register i64 x1 __asm__("x1") = b;
    register i64 x2 __asm__("x2") = c;
    register i64 x3 __asm__("x3") = d;
    register i64 x4 __asm__("x4") = e;
    register i64 x5 __asm__("x5") = f;
    __asm__ volatile("svc #0"
                     : "+r"(x0)
                     : "r"(x8), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x5)
                     : "memory", "cc");
    return x0;
}

i64 write(int fd, const void *p, u64 n) { return svc(64, fd, (i64)p, (i64)n, 0, 0, 0); }

i64 read(int fd, void *p, u64 n) { return svc(63, fd, (i64)p, (i64)n, 0, 0, 0); }

void exit(int code) {
    svc(94, code, 0, 0, 0, 0, 0);
    for (;;) {
    }
}

void _exit(int code) { exit(code); }

static unsigned char *heap;
static u64 heap_at;
static u64 heap_cap;

static void *map_heap(u64 n) {
    i64 p = svc(222, 0, (i64)n, 3, 0x22, -1, 0);
    if (p < 0) {
        return 0;
    }
    return (void *)p;
}

static void *bump(u64 n) {
    u64 need;
    unsigned char *p;
    if (heap == 0) {
        heap_cap = 64ull << 20;
        heap = map_heap(heap_cap);
        if (heap == 0) {
            return 0;
        }
    }
    need = (n + 15ull) & ~15ull;
    if (heap_at + need > heap_cap) {
        return 0;
    }
    p = heap + heap_at;
    heap_at += need;
    return p;
}

void *malloc(u64 n) {
    unsigned char *b = bump(16 + n);
    if (b == 0) {
        return 0;
    }
    *(u64 *)b = n;
    return b + 16;
}

void free(void *p) { (void)p; }

void *realloc(void *p, u64 n) {
    unsigned char *old;
    unsigned char *q;
    u64 have;
    u64 i;
    if (p == 0) {
        return malloc(n);
    }
    q = malloc(n);
    if (q == 0) {
        return 0;
    }
    old = (unsigned char *)p - 16;
    have = *(u64 *)old;
    if (have > n) {
        have = n;
    }
    for (i = 0; i < have; i++) {
        q[i] = ((unsigned char *)p)[i];
    }
    return q;
}

void *memcpy(void *d, const void *s, u64 n) {
    unsigned char *dd = d;
    const unsigned char *ss = s;
    u64 i;
    for (i = 0; i < n; i++) {
        dd[i] = ss[i];
    }
    return d;
}

void *memmove(void *d, const void *s, u64 n) {
    unsigned char *dd = d;
    const unsigned char *ss = s;
    u64 i;
    if (dd == (unsigned char *)s || n == 0) {
        return d;
    }
    if (dd < ss) {
        return memcpy(d, s, n);
    }
    i = n;
    while (i > 0) {
        i--;
        dd[i] = ss[i];
    }
    return d;
}

void *memset(void *d, int c, u64 n) {
    unsigned char *dd = d;
    u64 i;
    for (i = 0; i < n; i++) {
        dd[i] = (unsigned char)c;
    }
    return d;
}

int memcmp(const void *a, const void *b, u64 n) {
    const unsigned char *aa = a;
    const unsigned char *bb = b;
    u64 i;
    for (i = 0; i < n; i++) {
        if (aa[i] != bb[i]) {
            return aa[i] < bb[i] ? -1 : 1;
        }
    }
    return 0;
}

u64 strlen(const char *s) {
    u64 n = 0;
    if (s == 0) {
        return 0;
    }
    while (s[n] != 0) {
        n++;
    }
    return n;
}

void llr_trace_push(void *slot) { (void)slot; }

void llr_trace_pop(void *slot) { (void)slot; }

void llr_panic_frames(void) {}

int main(int argc, char **argv);

__attribute__((naked)) void _start(void) {
    __asm__ volatile("ldr x0, [sp]\n"
                     "add x1, sp, #8\n"
                     "bl main\n"
                     "mov x8, #94\n"
                     "svc #0\n");
}
