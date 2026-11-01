/* Android window, looper, and present. Input and insets live in llr_ui_android_in.c. */
#include "llr_ui_android.h"

#include <android/native_window.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static llr_and *host_of(void *p) {
    return (llr_and *)p;
}

static void negotiate(llr_and *h, ANativeWindow *win) {
    int32_t fmt = ANativeWindow_getFormat(win);
    if (fmt != WINDOW_FORMAT_RGBA_8888 && fmt != WINDOW_FORMAT_RGBX_8888 && fmt != 5) {
        ANativeWindow_setBuffersGeometry(win, 0, 0, WINDOW_FORMAT_RGBA_8888);
        fmt = WINDOW_FORMAT_RGBA_8888;
    }
    h->swizzle = fmt != 5;
    h->fmt_set = 1;
}

static void blit_row(uint8_t *d, const uint8_t *s, int n, int swizzle) {
    int x;
    if (!swizzle) {
        memcpy(d, s, (size_t)n * 4);
        return;
    }
    for (x = 0; x < n; x++) {
        d[x * 4] = s[x * 4 + 2];
        d[x * 4 + 1] = s[x * 4 + 1];
        d[x * 4 + 2] = s[x * 4];
        d[x * 4 + 3] = s[x * 4 + 3];
    }
}

static void blit_at(ANativeWindow_Buffer *buf, int x0, int y0, const uint8_t *px, int w, int hgt, int stride, int swizzle) {
    int y;
    int dw;
    int dh;
    uint8_t *base;
    if (x0 < 0) {
        x0 = 0;
    }
    if (y0 < 0) {
        y0 = 0;
    }
    dw = buf->width - x0;
    dh = buf->height - y0;
    if (w < dw) {
        dw = w;
    }
    if (hgt < dh) {
        dh = hgt;
    }
    if (dw < 1 || dh < 1 || !buf->bits) {
        return;
    }
    base = (uint8_t *)buf->bits + ((size_t)y0 * (size_t)buf->stride + (size_t)x0) * 4;
    for (y = 0; y < dh; y++) {
        uint8_t *d = base + (size_t)y * (size_t)buf->stride * 4;
        const uint8_t *s = px + (size_t)y * (size_t)stride;
        blit_row(d, s, dw, swizzle);
    }
}

static void drain(llr_and *h) {
    int events = 0;
    struct android_poll_source *source = NULL;
    int ident;
    while ((ident = ALooper_pollOnce(0, NULL, &events, (void **)&source)) >= 0) {
        if (source) {
            source->process(h->app, source);
        }
        source = NULL;
    }
}

static void pump(llr_and *h) {
    int events = 0;
    struct android_poll_source *source = NULL;
    int ident = ALooper_pollOnce(-1, NULL, &events, (void **)&source);
    if (ident >= 0 && source) {
        source->process(h->app, source);
    }
}

void *llr_ui_open(int64_t w, int64_t hgt) {
    llr_and *h;
    (void)w;
    (void)hgt;
    if (!llr_android_app) {
        return NULL;
    }
    h = calloc(1, sizeof *h);
    if (!h) {
        return NULL;
    }
    h->app = llr_android_app;
    h->first = 1;
    h->scale = 1.0f;
    h->log_w = 1;
    h->log_h = 1;
    h->app->userData = h;
    h->app->onAppCmd = llr_and_cmd;
    h->app->onInputEvent = llr_and_input;
    h->scale = llr_and_density(h);
    if (h->app->window) {
        llr_and_insets(h);
    }
    return h;
}

void llr_ui_close(void *p) {
    llr_and *h = host_of(p);
    if (!h) {
        return;
    }
    if (h->app && h->app->userData == h) {
        h->app->onAppCmd = NULL;
        h->app->onInputEvent = NULL;
        h->app->userData = NULL;
    }
    h->dead = 1;
    free(h);
}

void llr_ui_quit(void *p) {
    llr_and *h = host_of(p);
    if (!h) {
        return;
    }
    if (h->app && h->app->activity) {
        ANativeActivity_finish(h->app->activity);
    }
    llr_ui_close(p);
}

void llr_ui_size(void *p, int64_t *w, int64_t *hgt) {
    llr_and *h = host_of(p);
    if (!h) {
        return;
    }
    if (w) {
        *w = h->log_w > 0 ? h->log_w : 1;
    }
    if (hgt) {
        *hgt = h->log_h > 0 ? h->log_h : 1;
    }
}

float llr_ui_scale(void *p) {
    llr_and *h = host_of(p);
    if (!h || h->scale < 0.01f) {
        return 1.0f;
    }
    return h->scale;
}

void llr_ui_present(void *p, const uint8_t *px, int64_t w, int64_t hgt, int64_t stride) {
    llr_and *h = host_of(p);
    ANativeWindow *win;
    ANativeWindow_Buffer buf;
    if (!h || h->dead || !px || w <= 0 || hgt <= 0 || stride <= 0) {
        return;
    }
    win = h->app ? h->app->window : NULL;
    if (!win) {
        return;
    }
    if (!h->fmt_set) {
        negotiate(h, win);
    }
    if (ANativeWindow_lock(win, &buf, NULL) != 0) {
        return;
    }
    blit_at(&buf, h->il, h->it, px, (int)w, (int)hgt, (int)stride, h->swizzle);
    ANativeWindow_unlockAndPost(win);
}

int64_t llr_ui_poll(void *p, uint8_t *buf, int64_t cap) {
    llr_and *h = host_of(p);
    if (!h || !h->app) {
        return 0;
    }
    drain(h);
    if (h->first) {
        h->first = 0;
        llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
        return llr_ui_copy(h->q, &h->qn, buf, cap);
    }
    if (h->qn == 0 && !h->frame && !h->dead) {
        pump(h);
        drain(h);
    }
    if (h->app->destroyRequested) {
        llr_and_life(h, LLR_UI_DESTROY);
    }
    if (h->frame && !h->dead) {
        h->frame = 0;
        llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
    }
    return llr_ui_copy(h->q, &h->qn, buf, cap);
}

void llr_ui_request_frame(void *p) {
    llr_and *h = host_of(p);
    if (h) {
        h->frame = 1;
    }
}

int64_t llr_ui_now_ms(void *p) {
    struct timespec ts;
    (void)p;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
}

void llr_ui_set_cursor(void *p, int64_t id) {
    (void)p;
    (void)id;
}
