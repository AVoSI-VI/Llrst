#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

enum { N = 800000 };

int main(void) {
    char *buf = (char *)malloc(16 * (size_t)N);
    int64_t len = 0;
    int64_t acc = 0;
    int64_t i;
    int64_t k;
    if (buf == 0) {
        return 1;
    }
    for (i = 0; i < N; i++) {
        char tmp[32];
        int n;
        int c;
        if (i > 0) {
            buf[len++] = ',';
        }
        n = snprintf(tmp, sizeof tmp, "%lld", (long long)i);
        for (c = 0; c < n; c++) {
            buf[len++] = tmp[c];
        }
    }
    for (k = 0; k < len; k++) {
        acc = (acc * 131 + (unsigned char)buf[k]) & 0xffffffffLL;
    }
    printf("%lld %lld\n", (long long)len, (long long)acc);
    free(buf);
    return 0;
}
