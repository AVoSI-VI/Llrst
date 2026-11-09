#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { N = 800000, CAP = 2097152 };

struct slot {
    int64_t key;
    int64_t val;
    int used;
};

static int64_t lcg(int64_t x) { return (x * 1664525 + 1013904223) & 0xffffffffLL; }

static void put(struct slot *tab, int64_t key, int64_t val) {
    uint64_t i = (uint64_t)key & (CAP - 1);
    while (tab[i].used && tab[i].key != key) {
        i = (i + 1) & (CAP - 1);
    }
    tab[i].used = 1;
    tab[i].key = key;
    tab[i].val = val;
}

static int find(struct slot *tab, int64_t key, int64_t *out) {
    uint64_t i = (uint64_t)key & (CAP - 1);
    while (tab[i].used) {
        if (tab[i].key == key) {
            *out = tab[i].val;
            return 1;
        }
        i = (i + 1) & (CAP - 1);
    }
    return 0;
}

int main(void) {
    struct slot *tab = (struct slot *)calloc((size_t)CAP, sizeof(struct slot));
    int64_t x = 1;
    int64_t found = 0;
    int64_t total = 0;
    int64_t len = 0;
    int64_t i;
    if (tab == 0) {
        return 1;
    }
    for (i = 0; i < N; i++) {
        uint64_t h;
        x = lcg(x);
        h = (uint64_t)x & (CAP - 1);
        while (tab[h].used && tab[h].key != x) {
            h = (h + 1) & (CAP - 1);
        }
        if (!tab[h].used) {
            len++;
        }
        put(tab, x, i);
    }
    x = 1;
    for (i = 0; i < N; i++) {
        int64_t val = 0;
        x = lcg(x);
        if (find(tab, x, &val)) {
            found++;
            total += val;
        }
    }
    printf("%lld %lld %lld\n", (long long)found, (long long)total, (long long)len);
    free(tab);
    return 0;
}
