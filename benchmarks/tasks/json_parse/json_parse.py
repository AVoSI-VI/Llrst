import json

N = 800000


def main():
    parts = []
    i = 0
    while i < N:
        parts.append('{"id":%d,"ok":true}' % i)
        i += 1
    text = "[" + ",".join(parts) + "]"
    values = json.loads(text)
    total = 0
    for obj in values:
        total += obj["id"]
    print(total, len(values))


if __name__ == "__main__":
    main()
