#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

enum { MIN_D = 4, MAX_D = 13 };

struct node {
    struct node *left;
    struct node *right;
};

static struct node *make(int depth) {
    struct node *n = (struct node *)malloc(sizeof(struct node));
    if (depth == 0) {
        n->left = 0;
        n->right = 0;
    } else {
        n->left = make(depth - 1);
        n->right = make(depth - 1);
    }
    return n;
}

static int64_t check(const struct node *n) {
    if (n->left == 0) {
        return 1;
    }
    return 1 + check(n->left) + check(n->right);
}

static void release(struct node *n) {
    if (n == 0) {
        return;
    }
    release(n->left);
    release(n->right);
    free(n);
}

int main(void) {
    struct node *stretch = make(MAX_D + 1);
    struct node *long_lived;
    int64_t stretch_n = check(stretch);
    int64_t total = 0;
    int depth;
    release(stretch);
    long_lived = make(MAX_D);
    for (depth = MIN_D; depth <= MAX_D; depth += 2) {
        int iterations = 1 << (MAX_D - depth + MIN_D);
        int64_t acc = 0;
        int i;
        for (i = 0; i < iterations; i++) {
            struct node *t = make(depth);
            acc += check(t);
            release(t);
        }
        total += acc;
    }
    total += check(long_lived);
    release(long_lived);
    printf("%lld %lld\n", (long long)stretch_n, (long long)total);
    return 0;
}
