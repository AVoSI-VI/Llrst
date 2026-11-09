#include <stdint.h>
#include <stdio.h>

enum { STEPS = 50000, BODIES = 5 };

static const double PI = 3.141592653589793;
static const double DAYS = 365.24;
static const double DT = 0.01;

static double nsqrt(double x) {
    double guess;
    int i;
    if (x == 0.0) {
        return 0.0;
    }
    guess = x > 1.0 ? x * 0.5 : 1.0;
    for (i = 0; i < 8; i++) {
        guess = 0.5 * (guess + x / guess);
    }
    return guess;
}

static void advance(double b[][7], double dt) {
    int i, j;
    for (i = 0; i < BODIES; i++) {
        for (j = i + 1; j < BODIES; j++) {
            double dx = b[i][0] - b[j][0];
            double dy = b[i][1] - b[j][1];
            double dz = b[i][2] - b[j][2];
            double dist = nsqrt(dx * dx + dy * dy + dz * dz);
            double mag = dt / (dist * dist * dist);
            b[i][3] -= dx * b[j][6] * mag;
            b[i][4] -= dy * b[j][6] * mag;
            b[i][5] -= dz * b[j][6] * mag;
            b[j][3] += dx * b[i][6] * mag;
            b[j][4] += dy * b[i][6] * mag;
            b[j][5] += dz * b[i][6] * mag;
        }
    }
    for (i = 0; i < BODIES; i++) {
        b[i][0] += dt * b[i][3];
        b[i][1] += dt * b[i][4];
        b[i][2] += dt * b[i][5];
    }
}

static double energy(double b[][7]) {
    double e = 0.0;
    int i, j;
    for (i = 0; i < BODIES; i++) {
        e += 0.5 * b[i][6] * (b[i][3] * b[i][3] + b[i][4] * b[i][4] + b[i][5] * b[i][5]);
        for (j = i + 1; j < BODIES; j++) {
            double dx = b[i][0] - b[j][0];
            double dy = b[i][1] - b[j][1];
            double dz = b[i][2] - b[j][2];
            e -= b[i][6] * b[j][6] / nsqrt(dx * dx + dy * dy + dz * dz);
        }
    }
    return e;
}

int main(void) {
    double sm = 4.0 * PI * PI;
    double b[BODIES][7] = {
        {0, 0, 0, 0, 0, 0, 0},
        {4.84143144246472090, -1.16032004402742839, -0.103622044471123109, 0.00166007664274403694 * DAYS,
            0.00769901118419740425 * DAYS, -0.0000690460016972063023 * DAYS, 0.000954791938424326609},
        {8.34336671824457987, 4.12479856412430479, -0.403523417114321381, -0.00276742510726862411 * DAYS,
            0.00499852801234917238 * DAYS, 0.0000230417297573763929 * DAYS, 0.000285885980666130812},
        {12.8943695621391310, -15.1111514016986312, -0.223307578892655734, 0.00296460137564761618 * DAYS,
            0.00237847173959480950 * DAYS, -0.0000296589568540237556 * DAYS, 0.0000436624404335156298},
        {15.3796971148509165, -25.9193146099879641, 0.179258772950371181, 0.00268067772490389322 * DAYS,
            0.00162824170038242295 * DAYS, -0.0000951592254519715870 * DAYS, 0.0000515138902046611451},
    };
    double px = 0, py = 0, pz = 0;
    int i, s;
    b[0][6] = sm;
    for (i = 1; i < BODIES; i++) {
        b[i][6] *= sm;
    }
    for (i = 0; i < BODIES; i++) {
        px += b[i][3] * b[i][6];
        py += b[i][4] * b[i][6];
        pz += b[i][5] * b[i][6];
    }
    b[0][3] = -px / sm;
    b[0][4] = -py / sm;
    b[0][5] = -pz / sm;
    for (s = 0; s < STEPS; s++) {
        advance(b, DT);
    }
    printf("%lld\n", (long long)(energy(b) * 10000.0));
    return 0;
}
