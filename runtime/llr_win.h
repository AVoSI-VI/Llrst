/* Shared fd table and helpers for the Windows POSIX shims. */
#ifndef LLR_WIN_H
#define LLR_WIN_H

#define _CRT_DECLARE_NONSTDC_NAMES 0
#define _CRT_SECURE_NO_WARNINGS
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <errno.h>
#include <fcntl.h>
#include <io.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef fileno
#undef fileno
#endif
#ifdef popen
#undef popen
#endif
#ifdef pclose
#undef pclose
#endif
#ifdef isatty
#undef isatty
#endif
#ifdef mkdir
#undef mkdir
#endif
#ifdef getcwd
#undef getcwd
#endif

#define FD_BASE 256
/* Client threads share this table with the server. http_many holds 3072 of each. */
#define FD_MAX 8192
#define KIND_FILE 1
#define KIND_SOCK 2
#define L_CREAT 64
#define L_TRUNC 512
#define L_APPEND 1024
#define L_NONBLOCK 2048
#define F_GETFL 3
#define F_SETFL 4
#define PATH_CAP 4096

#define SOCK_BAD (~(uint64_t)0)

int llr_wsa_startup(void);
int llr_wsa_last_error(void);
int llr_wsa_close(uint64_t sock);
int llr_wsa_nonblock(uint64_t sock, int on);
uint64_t llr_wsa_socket(int domain, int ty, int proto);
int llr_wsa_bind(uint64_t sock, const void *addr, int len);
int llr_wsa_listen(uint64_t sock, int backlog);
int llr_wsa_connect(uint64_t sock, const void *addr, int len);
int llr_wsa_getsockname(uint64_t sock, void *addr, int *len);
uint64_t llr_wsa_accept(uint64_t sock, void *addr, int *len);
int llr_wsa_send(uint64_t sock, const void *buf, int n, int flags);
int llr_wsa_recv(uint64_t sock, void *buf, int n, int flags);
int llr_wsa_poll(void *fds, uint32_t n, int timeout);
short llr_wsa_to_events(short events);
short llr_wsa_from_events(short events);

typedef struct slot {
    int kind;
    HANDLE handle;
    uint64_t sock;
    int nonblock;
    int listening;
} Slot;

extern Slot slots[FD_MAX];
extern CRITICAL_SECTION fd_lock;

void ensure_init(void);
void set_errno(int err);
void map_wsa(void);
int take_new(int kind, HANDLE handle, uint64_t sock);
int copy_slot(int fd, Slot *out);
int take_slot(int fd, Slot *out);
HANDLE std_handle(int fd);

#endif
