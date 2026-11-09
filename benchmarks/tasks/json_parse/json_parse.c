#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

enum { N = 800000 };

static const char *at;
static int64_t sum;
static int64_t nums;

static void skip(void) {
    while (*at == ' ' || *at == '\n' || *at == '\r' || *at == '\t') {
        at++;
    }
}

static void parse_str(void) {
    at++;
    while (*at && *at != '"') {
        if (*at == '\\') {
            at++;
        }
        if (*at) {
            at++;
        }
    }
    if (*at == '"') {
        at++;
    }
}

static void parse_val(void);

static void parse_num(void) {
    int neg = 0;
    int64_t v = 0;
    if (*at == '-') {
        neg = 1;
        at++;
    }
    while (*at >= '0' && *at <= '9') {
        v = v * 10 + (*at - '0');
        at++;
    }
    if (*at == '.') {
        at++;
        while (*at >= '0' && *at <= '9') {
            at++;
        }
    }
    if (*at == 'e' || *at == 'E') {
        at++;
        if (*at == '+' || *at == '-') {
            at++;
        }
        while (*at >= '0' && *at <= '9') {
            at++;
        }
    }
    if (!neg) {
        sum += v;
        nums++;
    }
}

static void parse_val(void) {
    skip();
    if (*at == '{') {
        at++;
        skip();
        if (*at == '}') {
            at++;
            return;
        }
        for (;;) {
            parse_str();
            skip();
            at++;
            parse_val();
            skip();
            if (*at == ',') {
                at++;
                continue;
            }
            break;
        }
        at++;
        return;
    }
    if (*at == '[') {
        at++;
        skip();
        if (*at == ']') {
            at++;
            return;
        }
        for (;;) {
            parse_val();
            skip();
            if (*at == ',') {
                at++;
                continue;
            }
            break;
        }
        at++;
        return;
    }
    if (*at == '"') {
        parse_str();
        return;
    }
    if (*at == 't') {
        at += 4;
        return;
    }
    if (*at == 'f') {
        at += 5;
        return;
    }
    if (*at == 'n') {
        at += 4;
        return;
    }
    parse_num();
}

int main(void) {
    char *buf = (char *)malloc(32 * (size_t)N);
    int64_t len = 0;
    int64_t i;
    if (buf == 0) {
        return 1;
    }
    buf[len++] = '[';
    for (i = 0; i < N; i++) {
        char tmp[64];
        int n;
        int c;
        if (i > 0) {
            buf[len++] = ',';
        }
        n = snprintf(tmp, sizeof tmp, "{\"id\":%lld,\"ok\":true}", (long long)i);
        for (c = 0; c < n; c++) {
            buf[len++] = tmp[c];
        }
    }
    buf[len++] = ']';
    buf[len] = 0;
    at = buf;
    parse_val();
    printf("%lld %lld\n", (long long)sum, (long long)nums);
    free(buf);
    return 0;
}
