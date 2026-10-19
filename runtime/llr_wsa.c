/* Winsock calls live here so llr_win.c can define the POSIX names socket,
 * bind, and the rest. The DLL is loaded by name: linking ws2_32.lib would
 * also define those names and the link would fail. */
#define WIN32_LEAN_AND_MEAN
#define _WIN32_WINNT 0x0601
#include <winsock2.h>
#include <ws2tcpip.h>
#include <stdint.h>

typedef char llr_wsa_poll_size[(sizeof(WSAPOLLFD) == 16) ? 1 : -1];

typedef int(WSAAPI *wsa_startup_fn)(WORD, WSADATA *);
typedef int(WSAAPI *wsa_error_fn)(void);
typedef int(WSAAPI *wsa_close_fn)(SOCKET);
typedef int(WSAAPI *wsa_ioctl_fn)(SOCKET, long, u_long *);
typedef SOCKET(WSAAPI *wsa_socket_fn)(int, int, int);
typedef int(WSAAPI *wsa_bind_fn)(SOCKET, const struct sockaddr *, int);
typedef int(WSAAPI *wsa_listen_fn)(SOCKET, int);
typedef int(WSAAPI *wsa_connect_fn)(SOCKET, const struct sockaddr *, int);
typedef int(WSAAPI *wsa_name_fn)(SOCKET, struct sockaddr *, int *);
typedef SOCKET(WSAAPI *wsa_accept_fn)(SOCKET, struct sockaddr *, int *);
typedef int(WSAAPI *wsa_send_fn)(SOCKET, const char *, int, int);
typedef int(WSAAPI *wsa_recv_fn)(SOCKET, char *, int, int);
typedef int(WSAAPI *wsa_poll_fn)(WSAPOLLFD *, ULONG, INT);

static HMODULE dll;
static wsa_startup_fn p_startup;
static wsa_error_fn p_error;
static wsa_close_fn p_close;
static wsa_ioctl_fn p_ioctl;
static wsa_socket_fn p_socket;
static wsa_bind_fn p_bind;
static wsa_listen_fn p_listen;
static wsa_connect_fn p_connect;
static wsa_name_fn p_name;
static wsa_accept_fn p_accept;
static wsa_send_fn p_send;
static wsa_recv_fn p_recv;
static wsa_poll_fn p_poll;

static int load(void) {
    if (dll != NULL) {
        return 0;
    }
    dll = LoadLibraryA("ws2_32.dll");
    if (dll == NULL) {
        return -1;
    }
    p_startup = (wsa_startup_fn)GetProcAddress(dll, "WSAStartup");
    p_error = (wsa_error_fn)GetProcAddress(dll, "WSAGetLastError");
    p_close = (wsa_close_fn)GetProcAddress(dll, "closesocket");
    p_ioctl = (wsa_ioctl_fn)GetProcAddress(dll, "ioctlsocket");
    p_socket = (wsa_socket_fn)GetProcAddress(dll, "socket");
    p_bind = (wsa_bind_fn)GetProcAddress(dll, "bind");
    p_listen = (wsa_listen_fn)GetProcAddress(dll, "listen");
    p_connect = (wsa_connect_fn)GetProcAddress(dll, "connect");
    p_name = (wsa_name_fn)GetProcAddress(dll, "getsockname");
    p_accept = (wsa_accept_fn)GetProcAddress(dll, "accept");
    p_send = (wsa_send_fn)GetProcAddress(dll, "send");
    p_recv = (wsa_recv_fn)GetProcAddress(dll, "recv");
    p_poll = (wsa_poll_fn)GetProcAddress(dll, "WSAPoll");
    if (p_startup == NULL || p_error == NULL || p_close == NULL || p_ioctl == NULL || p_socket == NULL || p_bind == NULL
        || p_listen == NULL || p_connect == NULL || p_name == NULL || p_accept == NULL || p_send == NULL || p_recv == NULL
        || p_poll == NULL) {
        return -1;
    }
    return 0;
}

int llr_wsa_startup(void) {
    WSADATA data;
    if (load() != 0) {
        return -1;
    }
    return p_startup(MAKEWORD(2, 2), &data);
}

int llr_wsa_last_error(void) {
    if (p_error == NULL) {
        return 0;
    }
    return p_error();
}

int llr_wsa_close(uint64_t sock) {
    if (p_close == NULL) {
        return -1;
    }
    return p_close((SOCKET)sock);
}

int llr_wsa_nonblock(uint64_t sock, int on) {
    u_long nb = on ? 1 : 0;
    if (p_ioctl == NULL) {
        return -1;
    }
    return p_ioctl((SOCKET)sock, FIONBIO, &nb);
}

uint64_t llr_wsa_socket(int domain, int ty, int proto) {
    if (load() != 0) {
        return ~(uint64_t)0;
    }
    return (uint64_t)p_socket(domain, ty, proto);
}

int llr_wsa_bind(uint64_t sock, const void *addr, int len) {
    return p_bind((SOCKET)sock, (const struct sockaddr *)addr, len);
}

int llr_wsa_listen(uint64_t sock, int backlog) {
    return p_listen((SOCKET)sock, backlog);
}

int llr_wsa_connect(uint64_t sock, const void *addr, int len) {
    if (p_connect == NULL && load() != 0) {
        return -1;
    }
    return p_connect((SOCKET)sock, (const struct sockaddr *)addr, len);
}

int llr_wsa_getsockname(uint64_t sock, void *addr, int *len) {
    return p_name((SOCKET)sock, (struct sockaddr *)addr, len);
}

uint64_t llr_wsa_accept(uint64_t sock, void *addr, int *len) {
    return (uint64_t)p_accept((SOCKET)sock, (struct sockaddr *)addr, len);
}

int llr_wsa_send(uint64_t sock, const void *buf, int n, int flags) {
    return p_send((SOCKET)sock, (const char *)buf, n, flags);
}

int llr_wsa_recv(uint64_t sock, void *buf, int n, int flags) {
    return p_recv((SOCKET)sock, (char *)buf, n, flags);
}

int llr_wsa_poll(void *fds, uint32_t n, int timeout) {
    if (load() != 0) {
        return -1;
    }
    return p_poll((WSAPOLLFD *)fds, (ULONG)n, timeout);
}

short llr_wsa_to_events(short events) {
    short out = 0;
    if (events & 0x001) {
        out = (short)(out | POLLIN);
    }
    if (events & 0x004) {
        out = (short)(out | POLLOUT);
    }
    return out;
}

short llr_wsa_from_events(short events) {
    short out = 0;
    if (events & POLLIN) {
        out = (short)(out | 0x001);
    }
    if (events & POLLOUT) {
        out = (short)(out | 0x004);
    }
    if (events & POLLERR) {
        out = (short)(out | 0x008);
    }
    if (events & POLLHUP) {
        out = (short)(out | 0x010);
    }
    if (events & POLLNVAL) {
        out = (short)(out | 0x020);
    }
    return out;
}
