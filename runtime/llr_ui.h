/* Fixed 64-byte event record shared by every host shim.
 * Little-endian. kind: 1 pointer, 2 key, 3 text, 4 frame, 5 quit, 6 life, 7 inset.
 * Pointer a is 0 down, 1 move, 2 up, 3 cancel, 4 scroll. x y z w are logical pixels.
 * Key a is 1 down or 0 up, b is a Unicode scalar, c is shift|ctrl|alt|meta.
 * Text a is the byte count, and the bytes start at offset 32 (at most 32).
 * Life a is 0 pause, 1 resume, 2 destroy.
 * Inset x y z w are left, top, right, bottom logical pixels.
 * Action a is a bit (1 tap, 2 focus, 4 set value, 8 dismiss, 16 scroll), b is the node id,
 * and c is the set-value byte count. Those bytes start at offset 32. */
#ifndef LLR_UI_H
#define LLR_UI_H

#include <stdint.h>
#include <string.h>

enum {
    LLR_UI_POINTER = 1,
    LLR_UI_KEY = 2,
    LLR_UI_TEXT = 3,
    LLR_UI_FRAME = 4,
    LLR_UI_QUIT = 5,
    LLR_UI_LIFE = 6,
    LLR_UI_INSET = 7,
    LLR_UI_ACTION = 9
};

enum {
    LLR_UI_PAUSE = 0,
    LLR_UI_RESUME = 1,
    LLR_UI_DESTROY = 2
};

enum {
    LLR_UI_DOWN = 0,
    LLR_UI_MOVE = 1,
    LLR_UI_UP = 2,
    LLR_UI_CANCEL = 3,
    LLR_UI_SCROLL = 4
};

enum {
    LLR_UI_CUR_ARROW = 0,
    LLR_UI_CUR_TEXT = 1,
    LLR_UI_CUR_HAND = 2,
    LLR_UI_CUR_CROSS = 3,
    LLR_UI_CUR_WAIT = 4
};

#define LLR_UI_REC 64
#define LLR_UI_TEXTN 32
#define LLR_UI_RING 64

typedef struct llr_ui_ev {
    int32_t kind;
    int32_t a;
    int32_t b;
    int32_t c;
    float x;
    float y;
    float z;
    float w;
    uint8_t text[LLR_UI_TEXTN];
} llr_ui_ev;

_Static_assert(sizeof(llr_ui_ev) == LLR_UI_REC, "llr_ui record");

static inline void llr_ui_push(llr_ui_ev *q, int *n, llr_ui_ev ev) {
    if (*n > 0 && ev.kind == LLR_UI_POINTER && ev.a == LLR_UI_MOVE) {
        llr_ui_ev *prev = &q[*n - 1];
        if (prev->kind == LLR_UI_POINTER && prev->a == LLR_UI_MOVE) {
            *prev = ev;
            return;
        }
    }
    if (*n >= LLR_UI_RING) {
        memmove(q, q + 1, (LLR_UI_RING - 1) * sizeof(llr_ui_ev));
        *n = LLR_UI_RING - 1;
    }
    q[*n] = ev;
    *n += 1;
}

static inline void llr_ui_kind(llr_ui_ev *q, int *n, int kind) {
    llr_ui_ev ev;
    memset(&ev, 0, sizeof ev);
    ev.kind = kind;
    llr_ui_push(q, n, ev);
}

static inline void llr_ui_ptr(llr_ui_ev *q, int *n, int kind, float x, float y, float z, float w) {
    llr_ui_ev ev;
    memset(&ev, 0, sizeof ev);
    ev.kind = LLR_UI_POINTER;
    ev.a = kind;
    ev.x = x;
    ev.y = y;
    ev.z = z;
    ev.w = w;
    llr_ui_push(q, n, ev);
}

static inline void llr_ui_key(llr_ui_ev *q, int *n, int down, int code, int mods) {
    llr_ui_ev ev;
    memset(&ev, 0, sizeof ev);
    ev.kind = LLR_UI_KEY;
    ev.a = down;
    ev.b = code;
    ev.c = mods;
    llr_ui_push(q, n, ev);
}

static inline void llr_ui_text(llr_ui_ev *q, int *n, const void *bytes, int len) {
    const uint8_t *s = bytes;
    int off = 0;
    if (!s || len <= 0) {
        return;
    }
    while (off < len) {
        int m = len - off;
        llr_ui_ev ev;
        if (m > LLR_UI_TEXTN) {
            m = LLR_UI_TEXTN;
        }
        memset(&ev, 0, sizeof ev);
        ev.kind = LLR_UI_TEXT;
        ev.a = m;
        memcpy(ev.text, s + off, (size_t)m);
        llr_ui_push(q, n, ev);
        off += m;
    }
}

static inline int64_t llr_ui_copy(llr_ui_ev *q, int *n, uint8_t *buf, int64_t cap) {
    int take;
    int left;
    int64_t room;
    if (!q || !n || *n <= 0 || !buf || cap < LLR_UI_REC) {
        return 0;
    }
    room = cap / LLR_UI_REC;
    take = *n;
    if ((int64_t)take > room) {
        take = (int)room;
    }
    memcpy(buf, q, (size_t)take * sizeof(llr_ui_ev));
    left = *n - take;
    if (left > 0) {
        memmove(q, q + take, (size_t)left * sizeof(llr_ui_ev));
    }
    *n = left;
    return take;
}

static inline int llr_ui_utf8(uint32_t cp, uint8_t out[4]) {
    if (cp > 0x10FFFF || (cp >= 0xD800 && cp <= 0xDFFF)) {
        return 0;
    }
    if (cp < 0x80) {
        out[0] = (uint8_t)cp;
        return 1;
    }
    if (cp < 0x800) {
        out[0] = (uint8_t)(0xC0 | (cp >> 6));
        out[1] = (uint8_t)(0x80 | (cp & 0x3F));
        return 2;
    }
    if (cp < 0x10000) {
        out[0] = (uint8_t)(0xE0 | (cp >> 12));
        out[1] = (uint8_t)(0x80 | ((cp >> 6) & 0x3F));
        out[2] = (uint8_t)(0x80 | (cp & 0x3F));
        return 3;
    }
    out[0] = (uint8_t)(0xF0 | (cp >> 18));
    out[1] = (uint8_t)(0x80 | ((cp >> 12) & 0x3F));
    out[2] = (uint8_t)(0x80 | ((cp >> 6) & 0x3F));
    out[3] = (uint8_t)(0x80 | (cp & 0x3F));
    return 4;
}

void *llr_ui_open(int64_t w, int64_t h);
void llr_ui_close(void *p);
void llr_ui_quit(void *p);
void llr_ui_size(void *p, int64_t *w, int64_t *h);
float llr_ui_scale(void *p);
void llr_ui_present(void *p, const uint8_t *px, int64_t w, int64_t h, int64_t stride);
int64_t llr_ui_poll(void *p, uint8_t *buf, int64_t cap);
void llr_ui_request_frame(void *p);
int64_t llr_ui_now_ms(void *p);
void llr_ui_set_cursor(void *p, int64_t id);
int64_t llr_ui_clip_get(void *p, uint8_t *buf, int64_t cap);
void llr_ui_clip_set(void *p, const uint8_t *bytes, int64_t n);
void llr_ui_text_input(void *p, int64_t on, float x, float y, float w, float h);
void llr_ui_a11y_update(void *p, const uint8_t *bytes, int64_t n);

#endif
