/* Android host. Compiled with the NDK clang and android_native_app_glue. */
#ifndef LLR_UI_ANDROID_H
#define LLR_UI_ANDROID_H

#include "llr_ui.h"

#include <android_native_app_glue.h>

typedef struct llr_and {
    struct android_app *app;
    llr_ui_ev q[LLR_UI_RING];
    int qn;
    int first;
    int frame;
    int dead;
    int fmt_set;
    int swizzle;
    float scale;
    int log_w;
    int log_h;
    int il;
    int it;
    int ir;
    int ib;
} llr_and;

extern struct android_app *llr_android_app;

void llr_and_cmd(struct android_app *app, int32_t cmd);
int32_t llr_and_input(struct android_app *app, AInputEvent *ev);
void llr_and_insets(llr_and *h);
float llr_and_density(llr_and *h);

static inline void llr_and_life(llr_and *h, int a) {
    llr_ui_ev ev;
    if (!h) {
        return;
    }
    if (a == LLR_UI_DESTROY) {
        if (h->dead) {
            return;
        }
        h->dead = 1;
    }
    memset(&ev, 0, sizeof ev);
    ev.kind = LLR_UI_LIFE;
    ev.a = a;
    llr_ui_push(h->q, &h->qn, ev);
}

#endif
