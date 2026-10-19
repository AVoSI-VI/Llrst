/* Host stat(2) and dirent. Field offsets come from the platform headers, so the
 * same source is correct on Linux and macOS. */
#include <dirent.h>
#include <stdint.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

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

struct llr_frame {
    struct llr_frame *prev;
    const char *name;
};

extern struct llr_frame *llr_trace;

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

void llr_panic_frames(void) {
    struct llr_frame *f = llr_trace;
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
