#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

enum { N = 5000000 };

int main(void) {
    uint8_t *mark = (uint8_t *)malloc((size_t)N + 1);
    int64_t count = 0;
    int64_t last = 0;
    int64_t i;
    if (mark == 0) {
        return 1;
    }
    for (i = 0; i <= N; i++) {
        mark[i] = 1;
    }
    mark[0] = 0;
    mark[1] = 0;
    for (i = 2; i <= N; i++) {
        if (mark[i]) {
            int64_t j = i * i;
            count++;
            last = i;
            while (j <= N) {
                mark[j] = 0;
                j += i;
            }
        }
    }
    printf("%lld %lld\n", (long long)count, (long long)last);
    free(mark);
    return 0;
}
