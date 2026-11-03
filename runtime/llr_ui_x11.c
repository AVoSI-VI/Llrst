/* X11 surface. Scale is Xft.dpi / 96. Present is a 32-bit ZPixmap. */
#include "llr_ui_x11.h"

#include <stdlib.h>
#include <string.h>
#include <time.h>

float llr_ui_x11_div(llr_ui *h) {
    if (!h || h->scale < 0.01f) {
        return 1.0f;
    }
    return h->scale;
}

static int shift_of(unsigned long mask) {
    int s = 0;
    if (mask == 0) {
        return 0;
    }
    while ((mask & 1) == 0) {
        mask >>= 1;
        s++;
    }
    return s;
}

static float scale_of(Display *dpy) {
    char *rms = XResourceManagerString(dpy);
    const char *p;
    float dpi;
    if (!rms) {
        return 1.0f;
    }
    p = strstr(rms, "Xft.dpi");
    if (!p) {
        return 1.0f;
    }
    p += 7;
    while (*p == ' ' || *p == '\t' || *p == ':') {
        p++;
    }
    dpi = (float)atof(p);
    if (dpi < 1.0f) {
        return 1.0f;
    }
    return dpi / 96.0f;
}

static void note_masks(llr_ui *h) {
    Visual *v = DefaultVisual(h->dpy, h->screen);
    h->rs = shift_of(v->red_mask);
    h->gs = shift_of(v->green_mask);
    h->bs = shift_of(v->blue_mask);
    h->msb = ImageByteOrder(h->dpy) == MSBFirst;
    h->direct = !h->msb && v->red_mask == 0xFF0000 && v->green_mask == 0x00FF00 && v->blue_mask == 0x0000FF;
}

static void swizzle(llr_ui *h, const uint8_t *px, int w, int hgt, int stride) {
    size_t n = (size_t)w * (size_t)hgt * 4;
    int y;
    if (h->sw_n < n) {
        free(h->sw);
        h->sw = malloc(n);
        h->sw_n = h->sw ? n : 0;
    }
    if (!h->sw) {
        return;
    }
    for (y = 0; y < hgt; y++) {
        int x;
        for (x = 0; x < w; x++) {
            const uint8_t *s = px + (size_t)y * (size_t)stride + (size_t)x * 4;
            uint32_t pix = ((uint32_t)s[2] << h->rs) | ((uint32_t)s[1] << h->gs) | ((uint32_t)s[0] << h->bs);
            uint8_t *d = h->sw + ((size_t)y * (size_t)w + (size_t)x) * 4;
            if (h->msb) {
                d[0] = (uint8_t)(pix >> 24);
                d[1] = (uint8_t)(pix >> 16);
                d[2] = (uint8_t)(pix >> 8);
                d[3] = (uint8_t)pix;
            } else {
                d[0] = (uint8_t)pix;
                d[1] = (uint8_t)(pix >> 8);
                d[2] = (uint8_t)(pix >> 16);
                d[3] = (uint8_t)(pix >> 24);
            }
        }
    }
}

static void pump(llr_ui *h, int block) {
    XEvent ev;
    if (block && !XPending(h->dpy)) {
        XNextEvent(h->dpy, &ev);
        llr_ui_x11_handle(h, &ev);
    }
    while (XPending(h->dpy)) {
        XNextEvent(h->dpy, &ev);
        llr_ui_x11_handle(h, &ev);
    }
}

static void destroy(llr_ui *h) {
    int i;
    if (!h) {
        return;
    }
    if (h->ic) {
        XDestroyIC(h->ic);
    }
    if (h->im) {
        XCloseIM(h->im);
    }
    for (i = 0; i < 5; i++) {
        if (h->cursors[i]) {
            XFreeCursor(h->dpy, h->cursors[i]);
        }
    }
    if (h->gc) {
        XFreeGC(h->dpy, h->gc);
    }
    if (h->win) {
        XDestroyWindow(h->dpy, h->win);
    }
    if (h->dpy) {
        XCloseDisplay(h->dpy);
    }
    free(h->clip);
    free(h->sw);
    free(h);
}

void *llr_ui_open(int64_t w, int64_t hgt) {
    llr_ui *h = calloc(1, sizeof *h);
    int pw;
    int ph;
    if (!h) {
        return NULL;
    }
    if (w < 1) {
        w = 1;
    }
    if (hgt < 1) {
        hgt = 1;
    }
    h->dpy = XOpenDisplay(NULL);
    if (!h->dpy) {
        free(h);
        return NULL;
    }
    h->screen = DefaultScreen(h->dpy);
    h->scale = scale_of(h->dpy);
    h->logical_w = (int)w;
    h->logical_h = (int)hgt;
    h->first = 1;
    note_masks(h);
    pw = (int)(w * h->scale + 0.5);
    ph = (int)(hgt * h->scale + 0.5);
    if (pw < 1) {
        pw = 1;
    }
    if (ph < 1) {
        ph = 1;
    }
    h->win = XCreateSimpleWindow(h->dpy, RootWindow(h->dpy, h->screen), 0, 0, (unsigned)pw, (unsigned)ph, 0, 0, 0);
    if (!h->win) {
        XCloseDisplay(h->dpy);
        free(h);
        return NULL;
    }
    h->gc = XCreateGC(h->dpy, h->win, 0, NULL);
    h->wm_del = XInternAtom(h->dpy, "WM_DELETE_WINDOW", False);
    h->clip_atom = XInternAtom(h->dpy, "CLIPBOARD", False);
    h->utf8 = XInternAtom(h->dpy, "UTF8_STRING", False);
    h->targets = XInternAtom(h->dpy, "TARGETS", False);
    h->prop = XInternAtom(h->dpy, "LLR_CLIP", False);
    XSetWMProtocols(h->dpy, h->win, &h->wm_del, 1);
    XSelectInput(h->dpy, h->win, ExposureMask | StructureNotifyMask | KeyPressMask | KeyReleaseMask | ButtonPressMask | ButtonReleaseMask | PointerMotionMask);
    XStoreName(h->dpy, h->win, "llr");
    llr_ui_x11_init_input(h);
    XMapWindow(h->dpy, h->win);
    XFlush(h->dpy);
    return h;
}

void llr_ui_close(void *p) {
    destroy(p);
}

void llr_ui_quit(void *p) {
    destroy(p);
}

void llr_ui_size(void *p, int64_t *w, int64_t *h) {
    llr_ui *ui = p;
    if (!ui) {
        return;
    }
    if (w) {
        *w = ui->logical_w;
    }
    if (h) {
        *h = ui->logical_h;
    }
}

float llr_ui_scale(void *p) {
    llr_ui *h = p;
    if (!h) {
        return 1.0f;
    }
    h->scale = scale_of(h->dpy);
    return llr_ui_x11_div(h);
}

void llr_ui_present(void *p, const uint8_t *px, int64_t w, int64_t hgt, int64_t stride) {
    llr_ui *h = p;
    const uint8_t *src = px;
    int row = (int)stride;
    XImage *im;
    if (!h || !h->dpy || !px || w <= 0 || hgt <= 0) {
        return;
    }
    if (!h->direct) {
        swizzle(h, px, (int)w, (int)hgt, (int)stride);
        if (!h->sw) {
            return;
        }
        src = h->sw;
        row = (int)w * 4;
    }
    im = XCreateImage(h->dpy, DefaultVisual(h->dpy, h->screen), (unsigned)DefaultDepth(h->dpy, h->screen), ZPixmap, 0, (char *)src, (unsigned)w, (unsigned)hgt, 32, row);
    if (!im) {
        return;
    }
    XPutImage(h->dpy, h->win, h->gc, im, 0, 0, 0, 0, (unsigned)w, (unsigned)hgt);
    im->data = NULL;
    XDestroyImage(im);
    XFlush(h->dpy);
}

int64_t llr_ui_poll(void *p, uint8_t *buf, int64_t cap) {
    llr_ui *h = p;
    if (!h || !h->dpy) {
        return 0;
    }
    pump(h, 0);
    if (h->first) {
        h->first = 0;
        llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
        return llr_ui_copy(h->q, &h->qn, buf, cap);
    }
    if (h->qn == 0 && !h->frame) {
        pump(h, 1);
    }
    if (h->frame) {
        h->frame = 0;
        llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
    }
    return llr_ui_copy(h->q, &h->qn, buf, cap);
}

void llr_ui_request_frame(void *p) {
    llr_ui *h = p;
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

void llr_ui_a11y_update(void *p, const uint8_t *bytes, int64_t n) {
    (void)p;
    (void)bytes;
    (void)n;
}
