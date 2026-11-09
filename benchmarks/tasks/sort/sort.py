N = 1000000


def lcg(x):
    return (x * 1664525 + 1013904223) & 0xFFFFFFFF


def main():
    x = 1
    vals = []
    i = 0
    while i < N:
        x = lcg(x)
        vals.append(x)
        i += 1
    vals.sort()
    acc = 0
    step = N // 10
    i = 0
    while i < N:
        acc = (acc + vals[i] * (i + 1)) & 0x7FFFFFFF
        i += step
    print(vals[0], vals[N // 2], vals[-1], acc)


if __name__ == "__main__":
    main()
