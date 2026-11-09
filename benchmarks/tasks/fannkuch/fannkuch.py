N = 9


def flips(perm):
    q = perm[:]
    count = 0
    while q[0] != 0:
        k = q[0]
        i = 0
        j = k
        while i < j:
            q[i], q[j] = q[j], q[i]
            i += 1
            j -= 1
        count += 1
    return count


def main():
    perm = list(range(N))
    best = 0
    checksum = 0
    sign = 1

    def rec(k):
        nonlocal best, checksum, sign
        if k == 1:
            if perm[0] != 0:
                got = flips(perm)
                checksum += sign * got
                if got > best:
                    best = got
            sign = -sign
            return
        rec(k - 1)
        i = 0
        while i < k - 1:
            t = perm[0]
            j = 0
            while j < k - 1:
                perm[j] = perm[j + 1]
                j += 1
            perm[k - 1] = t
            rec(k - 1)
            i += 1

    rec(N)
    print(checksum, best)


if __name__ == "__main__":
    main()
