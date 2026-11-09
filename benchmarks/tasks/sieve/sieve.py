N = 5000000


def main():
    mark = bytearray(b"\x01") * (N + 1)
    mark[0] = 0
    mark[1] = 0
    count = 0
    last = 0
    i = 2
    while i <= N:
        if mark[i]:
            count += 1
            last = i
            j = i * i
            while j <= N:
                mark[j] = 0
                j += i
        i += 1
    print(count, last)


if __name__ == "__main__":
    main()
