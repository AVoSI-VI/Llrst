N = 800000


def main():
    text = ",".join(str(i) for i in range(N))
    acc = 0
    for byte in text.encode("ascii"):
        acc = (acc * 131 + byte) & 0xFFFFFFFF
    print(len(text), acc)


if __name__ == "__main__":
    main()
