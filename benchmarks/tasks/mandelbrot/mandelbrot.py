W = 600
LIM = 50


def main():
    count = 0
    y = 0
    while y < W:
        x = 0
        while x < W:
            cr = 2.0 * x / W - 1.5
            ci = 2.0 * y / W - 1.0
            zr = 0.0
            zi = 0.0
            inside = True
            k = 0
            while k < LIM:
                zr2 = zr * zr - zi * zi + cr
                zi = 2.0 * zr * zi + ci
                zr = zr2
                if zr * zr + zi * zi > 4.0:
                    inside = False
                    break
                k += 1
            if inside:
                count += 1
            x += 1
        y += 1
    print(count)


if __name__ == "__main__":
    main()
