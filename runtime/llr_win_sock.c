/* Socket half of the Windows POSIX shim. The fd table lives in llr_win.c. */
#include "llr_win.h"

static int sock_fd(uint64_t sock) {
    int fd = take_new(KIND_SOCK, NULL, sock);
    if (fd < 0) {
        llr_wsa_close(sock);
        set_errno(24);
    }
    return fd;
}

static uint64_t socket_of(int fd) {
    Slot slot;
    if (!copy_slot(fd, &slot) || slot.kind != KIND_SOCK) {
        set_errno(9);
        return SOCK_BAD;
    }
    return slot.sock;
}

int socket(int domain, int ty, int proto) {
    ensure_init();
    uint64_t sock = llr_wsa_socket(domain, ty, proto);
    if (sock == SOCK_BAD) {
        map_wsa();
        return -1;
    }
    return sock_fd(sock);
}

int bind(int fd, void *addr, uint32_t len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (llr_wsa_bind(sock, addr, (int)len) != 0) {
        map_wsa();
        return -1;
    }
    return 0;
}

int listen(int fd, int backlog) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (llr_wsa_listen(sock, backlog) != 0) {
        int again = 0;
        EnterCriticalSection(&fd_lock);
        if (fd >= FD_BASE && fd < FD_BASE + FD_MAX && slots[fd - FD_BASE].listening) {
            again = 1;
        }
        LeaveCriticalSection(&fd_lock);
        if (again) {
            return 0;
        }
        map_wsa();
        return -1;
    }
    EnterCriticalSection(&fd_lock);
    if (fd >= FD_BASE && fd < FD_BASE + FD_MAX) {
        slots[fd - FD_BASE].listening = 1;
    }
    LeaveCriticalSection(&fd_lock);
    return 0;
}

int accept(int fd, void *addr, uint32_t *len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    int got = len == NULL ? 0 : (int)*len;
    uint64_t client = llr_wsa_accept(sock, addr, len == NULL ? NULL : &got);
    if (client == SOCK_BAD) {
        map_wsa();
        return -1;
    }
    if (len != NULL) {
        *len = (uint32_t)got;
    }
    return sock_fd(client);
}

int connect(int fd, void *addr, uint32_t len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (llr_wsa_connect(sock, addr, (int)len) != 0) {
        map_wsa();
        return -1;
    }
    return 0;
}

int getsockname(int fd, void *addr, uint32_t *len) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    int got = len == NULL ? 0 : (int)*len;
    if (llr_wsa_getsockname(sock, addr, &got) != 0) {
        map_wsa();
        return -1;
    }
    if (len != NULL) {
        *len = (uint32_t)got;
    }
    return 0;
}

int64_t send(int fd, const void *buf, uint64_t n, int flags) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (n > 0x7fffffff) {
        n = 0x7fffffff;
    }
    int sent = llr_wsa_send(sock, buf, (int)n, flags);
    if (sent < 0) {
        map_wsa();
        return -1;
    }
    return sent;
}

int64_t recv(int fd, void *buf, uint64_t n, int flags) {
    ensure_init();
    uint64_t sock = socket_of(fd);
    if (sock == SOCK_BAD) {
        return -1;
    }
    if (n > 0x7fffffff) {
        n = 0x7fffffff;
    }
    int got = llr_wsa_recv(sock, buf, (int)n, flags);
    if (got < 0) {
        map_wsa();
        return -1;
    }
    return got;
}

struct lpoll {
    int32_t fd;
    int16_t events;
    int16_t revents;
};

struct wsa_poll {
    uint64_t sock;
    int16_t events;
    int16_t revents;
    int16_t pad1;
    int16_t pad2;
};

int poll(void *base, uint64_t n, int timeout) {
    ensure_init();
    if (n > 4096) {
        n = 4096;
    }
    struct lpoll *in = (struct lpoll *)base;
    struct wsa_poll *wsa = (struct wsa_poll *)calloc((size_t)n, sizeof(struct wsa_poll));
    if (n > 0 && wsa == NULL) {
        set_errno(12);
        return -1;
    }
    for (uint64_t i = 0; i < n; i++) {
        Slot slot;
        wsa[i].sock = copy_slot(in[i].fd, &slot) && slot.kind == KIND_SOCK ? slot.sock : SOCK_BAD;
        wsa[i].events = llr_wsa_to_events(in[i].events);
        wsa[i].revents = 0;
    }
    int ready = llr_wsa_poll(wsa, (uint32_t)n, timeout);
    if (ready < 0) {
        map_wsa();
        free(wsa);
        return -1;
    }
    for (uint64_t i = 0; i < n; i++) {
        in[i].revents = llr_wsa_from_events(wsa[i].revents);
    }
    free(wsa);
    return ready;
}

int fcntl(int fd, int cmd, int arg) {
    ensure_init();
    if (fd < FD_BASE || fd >= FD_BASE + FD_MAX) {
        set_errno(9);
        return -1;
    }
    EnterCriticalSection(&fd_lock);
    Slot *slot = &slots[fd - FD_BASE];
    if (slot->kind != KIND_SOCK) {
        LeaveCriticalSection(&fd_lock);
        set_errno(9);
        return -1;
    }
    if (cmd == F_GETFL) {
        int flags = slot->nonblock ? L_NONBLOCK : 0;
        LeaveCriticalSection(&fd_lock);
        return flags;
    }
    if (cmd != F_SETFL) {
        LeaveCriticalSection(&fd_lock);
        set_errno(22);
        return -1;
    }
    uint64_t sock = slot->sock;
    LeaveCriticalSection(&fd_lock);
    if (llr_wsa_nonblock(sock, (arg & L_NONBLOCK) ? 1 : 0) != 0) {
        map_wsa();
        return -1;
    }
    EnterCriticalSection(&fd_lock);
    slots[fd - FD_BASE].nonblock = (arg & L_NONBLOCK) ? 1 : 0;
    LeaveCriticalSection(&fd_lock);
    return 0;
}
