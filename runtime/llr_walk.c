/* Included by llr_stat.c. x86-64 SysV records frames without llr_trace_push. */
struct llr_fnspan {
    const char *start;
    unsigned len;
    unsigned pad;
    const char *name;
};

extern const struct llr_fnspan llr_fnspans[] __attribute__((weak));
extern const unsigned llr_fnspans_n __attribute__((weak));
extern const char llr_frame_walk __attribute__((weak));

static int span_hit(const void *pc, const char **name) {
    unsigned i;
    if (&llr_fnspans_n == 0) {
        return 0;
    }
    for (i = 0; i < llr_fnspans_n; i++) {
        const char *s = llr_fnspans[i].start;
        if ((const char *)pc >= s && (const char *)pc < s + llr_fnspans[i].len) {
            *name = llr_fnspans[i].name;
            return 1;
        }
    }
    return 0;
}

static void print_frame(const char *name, int *n) {
    unsigned long len = 0;
    if (*n >= 32 || name == 0 || skip_frame(name)) {
        return;
    }
    while (name[len] != 0) {
        len++;
    }
    write(2, "P|", 2);
    write(2, name, len);
    write(2, "\n", 1);
    (*n)++;
}

__attribute__((noinline, optimize("no-omit-frame-pointer"))) void llr_panic_frames(void) {
    struct llr_frame *f = trace_head();
    void **rbp;
    int n = 0;
    int steps = 0;
    int magic = &llr_frame_walk != 0;
    int spans = &llr_fnspans_n != 0;
    while (f != 0 && n < 32) {
        print_frame(f->name, &n);
        f = f->prev;
    }
    if (!magic && !spans) {
        return;
    }
    rbp = __builtin_frame_address(1);
    while (rbp && n < 32 && steps < 32) {
        void **next;
        steps++;
        if (((unsigned long)rbp & 7) != 0) {
            break;
        }
        if (magic) {
            if (*(unsigned long long *)((char *)rbp - 16) != 0x4C4C5254ull) {
                break;
            }
            print_frame(*(const char **)((char *)rbp - 8), &n);
        } else {
            const char *name = 0;
            if (!span_hit(*(void **)((char *)rbp + 8), &name)) {
                break;
            }
            print_frame(name, &n);
        }
        next = *rbp;
        if (next <= rbp) {
            break;
        }
        rbp = next;
    }
}
