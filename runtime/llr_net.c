/* Included by llr_stat.c so the link line stays one object. Linux sockaddr layout. */
#include <arpa/inet.h>
#include <netdb.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/time.h>

static const void *llr_host_addr(const void *addr, uint32_t len, unsigned char tmp[28]) {
#ifdef __APPLE__
    const unsigned char *s = (const unsigned char *)addr;
    if (s != 0 && len == 16 && s[0] == 2 && s[1] == 0) {
        memset(tmp, 0, 16);
        tmp[0] = 16;
        tmp[1] = 2;
        memcpy(tmp + 2, s + 2, 14);
        return tmp;
    }
    if (s != 0 && len == 28 && s[0] == 10 && s[1] == 0) {
        memcpy(tmp, s, 28);
        tmp[0] = 28;
        tmp[1] = 30; /* AF_INET6 */
        return tmp;
    }
#else
    (void)len;
    (void)tmp;
#endif
    return addr;
}

int64_t llr_sendto(int fd, const void *buf, uint64_t n, const void *addr, uint32_t len) {
    unsigned char tmp[28];
    const void *use = llr_host_addr(addr, len, tmp);
    ssize_t w = sendto(fd, buf, (size_t)n, 0, (const struct sockaddr *)use, (socklen_t)len);
    return (int64_t)w;
}

int64_t llr_recvfrom(int fd, void *buf, uint64_t n, void *addr, uint32_t *alen) {
    socklen_t got = alen == 0 ? 0 : (socklen_t)*alen;
    ssize_t r = recvfrom(fd, buf, (size_t)n, 0, (struct sockaddr *)addr, alen == 0 ? 0 : &got);
    if (alen != 0 && r >= 0) {
        *alen = (uint32_t)got;
    }
    return (int64_t)r;
}

int llr_set_timeout(int fd, int64_t ms) {
    struct timeval tv;
    tv.tv_sec = ms <= 0 ? 0 : (time_t)(ms / 1000);
    tv.tv_usec = ms <= 0 ? 0 : (suseconds_t)((ms % 1000) * 1000);
    if (setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof tv) != 0) {
        return -1;
    }
    if (setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof tv) != 0) {
        return -1;
    }
    return 0;
}

int llr_resolve(const char *host, char *buf, uint64_t cap, int64_t *out_n) {
    struct addrinfo hints;
    struct addrinfo *res = 0;
    struct addrinfo *p;
    char *w;
    uint64_t left;
    int64_t n = 0;
    if (out_n != 0) {
        *out_n = 0;
    }
    if (host == 0 || buf == 0 || cap == 0) {
        return -1;
    }
    memset(&hints, 0, sizeof hints);
    hints.ai_socktype = SOCK_STREAM;
    hints.ai_family = AF_UNSPEC;
    if (getaddrinfo(host, 0, &hints, &res) != 0) {
        return -1;
    }
    w = buf;
    left = cap;
    for (p = res; p != 0; p = p->ai_next) {
        char ip[64];
        void *src = 0;
        size_t k;
        if (p->ai_family == AF_INET) {
            src = &((struct sockaddr_in *)p->ai_addr)->sin_addr;
        } else if (p->ai_family == AF_INET6) {
            src = &((struct sockaddr_in6 *)p->ai_addr)->sin6_addr;
        } else {
            continue;
        }
        if (inet_ntop(p->ai_family, src, ip, sizeof ip) == 0) {
            continue;
        }
        k = strlen(ip) + 1;
        if (k > left) {
            break;
        }
        memcpy(w, ip, k);
        w += k;
        left -= k;
        n++;
    }
    freeaddrinfo(res);
    if (out_n != 0) {
        *out_n = n;
    }
    return n == 0 ? -1 : 0;
}
