#include <stdint.h>
#include <stdio.h>

enum { W = 600, LIM = 50 };

int main(void) {
    int64_t count = 0;
    int y;
    for (y = 0; y < W; y++) {
        int x;
        for (x = 0; x < W; x++) {
            double cr = 2.0 * x / W - 1.5;
            double ci = 2.0 * y / W - 1.0;
            double zr = 0.0;
            double zi = 0.0;
            int inside = 1;
            int k;
            for (k = 0; k < LIM; k++) {
                double zr2 = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = zr2;
                if (zr * zr + zi * zi > 4.0) {
                    inside = 0;
                    break;
                }
            }
            if (inside) {
                count++;
            }
        }
    }
    printf("%lld\n", (long long)count);
    return 0;
}
