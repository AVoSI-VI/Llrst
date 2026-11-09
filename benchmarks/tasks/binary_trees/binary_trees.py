MIN_D = 4
MAX_D = 13


class Node:
    __slots__ = ("left", "right")

    def __init__(self, depth):
        if depth == 0:
            self.left = None
            self.right = None
        else:
            self.left = Node(depth - 1)
            self.right = Node(depth - 1)


def check(node):
    if node.left is None:
        return 1
    return 1 + check(node.left) + check(node.right)


def main():
    stretch = check(Node(MAX_D + 1))
    long_lived = Node(MAX_D)
    total = 0
    depth = MIN_D
    while depth <= MAX_D:
        iterations = 1 << (MAX_D - depth + MIN_D)
        acc = 0
        i = 0
        while i < iterations:
            acc += check(Node(depth))
            i += 1
        total += acc
        depth += 2
    total += check(long_lived)
    print(stretch, total)


if __name__ == "__main__":
    main()
