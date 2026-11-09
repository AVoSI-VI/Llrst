STEPS = 50000
DT = 0.01
PI = 3.141592653589793
DAYS = 365.24


def nsqrt(x):
    if x == 0.0:
        return 0.0
    guess = x * 0.5 if x > 1.0 else 1.0
    i = 0
    while i < 8:
        guess = 0.5 * (guess + x / guess)
        i += 1
    return guess


def solar():
    return 4.0 * PI * PI


def bodies():
    sm = solar()
    return [
        [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, sm],
        [
            4.84143144246472090,
            -1.16032004402742839,
            -0.103622044471123109,
            0.00166007664274403694 * DAYS,
            0.00769901118419740425 * DAYS,
            -0.0000690460016972063023 * DAYS,
            0.000954791938424326609 * sm,
        ],
        [
            8.34336671824457987,
            4.12479856412430479,
            -0.403523417114321381,
            -0.00276742510726862411 * DAYS,
            0.00499852801234917238 * DAYS,
            0.0000230417297573763929 * DAYS,
            0.000285885980666130812 * sm,
        ],
        [
            12.8943695621391310,
            -15.1111514016986312,
            -0.223307578892655734,
            0.00296460137564761618 * DAYS,
            0.00237847173959480950 * DAYS,
            -0.0000296589568540237556 * DAYS,
            0.0000436624404335156298 * sm,
        ],
        [
            15.3796971148509165,
            -25.9193146099879641,
            0.179258772950371181,
            0.00268067772490389322 * DAYS,
            0.00162824170038242295 * DAYS,
            -0.0000951592254519715870 * DAYS,
            0.0000515138902046611451 * sm,
        ],
    ]


def offset(bs, sm):
    px = 0.0
    py = 0.0
    pz = 0.0
    for b in bs:
        px += b[3] * b[6]
        py += b[4] * b[6]
        pz += b[5] * b[6]
    bs[0][3] = -px / sm
    bs[0][4] = -py / sm
    bs[0][5] = -pz / sm


def advance(bs, dt):
    n = len(bs)
    i = 0
    while i < n:
        j = i + 1
        while j < n:
            p = bs[i]
            q = bs[j]
            dx = p[0] - q[0]
            dy = p[1] - q[1]
            dz = p[2] - q[2]
            dist = nsqrt(dx * dx + dy * dy + dz * dz)
            mag = dt / (dist * dist * dist)
            p[3] -= dx * q[6] * mag
            p[4] -= dy * q[6] * mag
            p[5] -= dz * q[6] * mag
            q[3] += dx * p[6] * mag
            q[4] += dy * p[6] * mag
            q[5] += dz * p[6] * mag
            j += 1
        i += 1
    for b in bs:
        b[0] += dt * b[3]
        b[1] += dt * b[4]
        b[2] += dt * b[5]


def energy(bs):
    e = 0.0
    n = len(bs)
    i = 0
    while i < n:
        p = bs[i]
        e += 0.5 * p[6] * (p[3] * p[3] + p[4] * p[4] + p[5] * p[5])
        j = i + 1
        while j < n:
            q = bs[j]
            dx = p[0] - q[0]
            dy = p[1] - q[1]
            dz = p[2] - q[2]
            e -= p[6] * q[6] / nsqrt(dx * dx + dy * dy + dz * dz)
            j += 1
        i += 1
    return e


def main():
    sm = solar()
    bs = bodies()
    offset(bs, sm)
    i = 0
    while i < STEPS:
        advance(bs, DT)
        i += 1
    print(int(energy(bs) * 10000.0))


if __name__ == "__main__":
    main()
