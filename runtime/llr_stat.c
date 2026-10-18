/* Host stat(2) and dirent. Field offsets come from the platform headers, so the
 * same source is correct on Linux and macOS. */
#include <dirent.h>
#include <stdint.h>
#include <sys/stat.h>

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
