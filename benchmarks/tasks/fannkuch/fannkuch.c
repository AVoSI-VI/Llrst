#include <stdint.h>
#include <stdio.h>
#include <string.h>

enum { N = 9 };

static int64_t flips(const int64_t *perm) {
    int64_t q[N];
    int64_t count = 0;
    memcpy(q, perm, sizeof q);
    while (q[0] != 0) {
        int64_t k = q[0];
        int64_t i = 0;
        int64_t j = k;
        while (i < j) {
            int64_t t = q[i];
            q[i] = q[j];
            q[j] = t;
            i++;
            j--;
        }
        count++;
    }
    return count;
}

static void rec(int64_t *perm, int64_t k, int64_t *best, int64_t *checksum, int64_t *sign) {
    if (k == 1) {
        if (perm[0] != 0) {
            int64_t got = flips(perm);
            *checksum += *sign * got;
            if (got > *best) {
                *best = got;
            }
        }
        *sign = -*sign;
        return;
    }
    rec(perm, k - 1, best, checksum, sign);
    {
        int64_t i = 0;
        while (i < k - 1) {
            int64_t t = perm[0];
            int64_t j = 0;
            while (j < k - 1) {
                perm[j] = perm[j + 1];
                j++;
            }
            perm[k - 1] = t;
            rec(perm, k - 1, best, checksum, sign);
            i++;
        }
    }
}

int main(void) {
    int64_t perm[N];
    int64_t best = 0;
    int64_t checksum = 0;
    int64_t sign = 1;
    int64_t i;
    for (i = 0; i < N; i++) {
        perm[i] = i;
    }
    rec(perm, N, &best, &checksum, &sign);
    printf("%lld %lld\n", (long long)checksum, (long long)best);
    return 0;
}
