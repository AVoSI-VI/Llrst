#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

enum { N = 1000000 };

static int cmp(const void *a, const void *b) {
    int64_t x = *(const int64_t *)a;
    int64_t y = *(const int64_t *)b;
    if (x < y) {
        return -1;
    }
    if (x > y) {
        return 1;
    }
    return 0;
}

int main(void) {
    int64_t *vals = (int64_t *)malloc(sizeof(int64_t) * (size_t)N);
    int64_t x = 1;
    int64_t acc = 0;
    int64_t i;
    int64_t step = N / 10;
    if (vals == 0) {
        return 1;
    }
    for (i = 0; i < N; i++) {
        x = (x * 1664525 + 1013904223) & 0xffffffffLL;
        vals[i] = x;
    }
    qsort(vals, (size_t)N, sizeof(int64_t), cmp);
    for (i = 0; i < N; i += step) {
        acc = (acc + vals[i] * (i + 1)) & 0x7fffffffLL;
    }
    printf("%lld %lld %lld %lld\n", (long long)vals[0], (long long)vals[N / 2], (long long)vals[N - 1],
        (long long)acc);
    free(vals);
    return 0;
}
