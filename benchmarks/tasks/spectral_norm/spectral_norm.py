N = 500
ITERS = 10


def aval(i, j):
    ij = i + j
    return 1.0 / ((ij * (ij + 1)) // 2 + i + 1)


def mul(v, trans):
    n = len(v)
    out = [0.0] * n
    i = 0
    while i < n:
        s = 0.0
        j = 0
        while j < n:
            if trans:
                s += aval(j, i) * v[j]
            else:
                s += aval(i, j) * v[j]
            j += 1
        out[i] = s
        i += 1
    return out


def main():
    u = [1.0] * N
    k = 0
    while k < ITERS:
        v = mul(u, False)
        u = mul(v, True)
        k += 1
    acc = 0
    for x in u:
        acc += int(x * 1000000.0)
    print(acc)


if __name__ == "__main__":
    main()
