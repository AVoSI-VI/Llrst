N = 800000


def lcg(x):
    return (x * 1664525 + 1013904223) & 0xFFFFFFFF


def main():
    table = {}
    x = 1
    i = 0
    while i < N:
        x = lcg(x)
        table[x] = i
        i += 1
    found = 0
    total = 0
    x = 1
    i = 0
    while i < N:
        x = lcg(x)
        if x in table:
            found += 1
            total += table[x]
        i += 1
    print(found, total, len(table))


if __name__ == "__main__":
    main()
