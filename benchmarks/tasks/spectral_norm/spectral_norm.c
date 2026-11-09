#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

enum { N = 500, ITERS = 10 };

static double aval(int64_t i, int64_t j) {
    int64_t ij = i + j;
    return 1.0 / (double)((ij * (ij + 1)) / 2 + i + 1);
}

static void mul(const double *v, double *out, int trans) {
    int64_t i;
    for (i = 0; i < N; i++) {
        double s = 0.0;
        int64_t j;
        for (j = 0; j < N; j++) {
            s += (trans ? aval(j, i) : aval(i, j)) * v[j];
        }
        out[i] = s;
    }
}

int main(void) {
    double *u = (double *)malloc(sizeof(double) * N);
    double *v = (double *)malloc(sizeof(double) * N);
    double *w = (double *)malloc(sizeof(double) * N);
    int64_t acc = 0;
    int k;
    int64_t i;
    for (i = 0; i < N; i++) {
        u[i] = 1.0;
    }
    for (k = 0; k < ITERS; k++) {
        mul(u, v, 0);
        mul(v, w, 1);
        for (i = 0; i < N; i++) {
            u[i] = w[i];
        }
    }
    for (i = 0; i < N; i++) {
        acc += (int64_t)(u[i] * 1000000.0);
    }
    printf("%lld\n", (long long)acc);
    free(u);
    free(v);
    free(w);
    return 0;
}
