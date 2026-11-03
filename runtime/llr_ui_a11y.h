/* Shared accessibility blob. Little-endian. count, then each node:
 * u32 id parent role flags actions, f32 x y w h, u32 n, n label bytes,
 * u32 n, n value bytes. Parent 0xFFFFFFFF is a root. */
#ifndef LLR_UI_A11Y_H
#define LLR_UI_A11Y_H

#include "llr_ui.h"

#include <stdlib.h>
#include <string.h>

typedef struct llr_a11y_node {
    uint32_t id;
    uint32_t parent;
    uint32_t role;
    uint32_t flags;
    uint32_t actions;
    float x, y, w, h;
    char *label;
    char *value;
} llr_a11y_node;

typedef struct llr_a11y_tree {
    llr_a11y_node *nodes;
    uint32_t count;
} llr_a11y_tree;

static inline uint32_t llr_a11y_u32(const uint8_t *p) {
    uint32_t n;
    memcpy(&n, p, 4);
    return n;
}

static inline float llr_a11y_f32(const uint8_t *p) {
    float n;
    memcpy(&n, p, 4);
    return n;
}

static inline char *llr_a11y_dup(const uint8_t *p, uint32_t n) {
    char *s = (char *)malloc((size_t)n + 1);
    if (!s) {
        return NULL;
    }
    if (n) {
        memcpy(s, p, n);
    }
    s[n] = 0;
    return s;
}

static inline void llr_a11y_clear(llr_a11y_tree *t) {
    uint32_t i;
    if (!t) {
        return;
    }
    for (i = 0; t->nodes && i < t->count; i++) {
        free(t->nodes[i].label);
        free(t->nodes[i].value);
    }
    free(t->nodes);
    t->nodes = NULL;
    t->count = 0;
}

static inline int llr_a11y_parse(llr_a11y_tree *t, const uint8_t *b, int64_t n) {
    uint32_t count;
    uint32_t i;
    int64_t at;
    llr_a11y_clear(t);
    if (!b || n < 4) {
        return 0;
    }
    count = llr_a11y_u32(b);
    if (count > 4096) {
        return 0;
    }
    t->nodes = count ? (llr_a11y_node *)calloc(count, sizeof(llr_a11y_node)) : NULL;
    if (count && !t->nodes) {
        return 0;
    }
    t->count = count;
    at = 4;
    for (i = 0; i < count; i++) {
        uint32_t ln;
        uint32_t vn;
        llr_a11y_node *node = &t->nodes[i];
        if (at + 40 > n) {
            llr_a11y_clear(t);
            return 0;
        }
        node->id = llr_a11y_u32(b + at);
        node->parent = llr_a11y_u32(b + at + 4);
        node->role = llr_a11y_u32(b + at + 8);
        node->flags = llr_a11y_u32(b + at + 12);
        node->actions = llr_a11y_u32(b + at + 16);
        node->x = llr_a11y_f32(b + at + 20);
        node->y = llr_a11y_f32(b + at + 24);
        node->w = llr_a11y_f32(b + at + 28);
        node->h = llr_a11y_f32(b + at + 32);
        ln = llr_a11y_u32(b + at + 36);
        at += 40;
        if (at + (int64_t)ln + 4 > n) {
            llr_a11y_clear(t);
            return 0;
        }
        node->label = llr_a11y_dup(b + at, ln);
        at += ln;
        vn = llr_a11y_u32(b + at);
        at += 4;
        if (at + (int64_t)vn > n || !node->label) {
            llr_a11y_clear(t);
            return 0;
        }
        node->value = llr_a11y_dup(b + at, vn);
        at += vn;
        if (!node->value) {
            llr_a11y_clear(t);
            return 0;
        }
    }
    return 1;
}

static inline void llr_ui_action(llr_ui_ev *q, int *n, int action, int id, const void *bytes, int len) {
    llr_ui_ev ev;
    memset(&ev, 0, sizeof ev);
    ev.kind = LLR_UI_ACTION;
    ev.a = action;
    ev.b = id;
    if (bytes && len > 0) {
        if (len > LLR_UI_TEXTN) {
            len = LLR_UI_TEXTN;
        }
        ev.c = len;
        memcpy(ev.text, bytes, (size_t)len);
    }
    llr_ui_push(q, n, ev);
}

#endif
