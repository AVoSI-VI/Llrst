/* X11 pointer, Xkb keysyms, CLIPBOARD selection, and cursor. */
#include "llr_ui_x11.h"

#include <X11/XKBlib.h>
#include <X11/cursorfont.h>
#include <locale.h>
#include <stdlib.h>
#include <string.h>
#include <sys/select.h>

static int mods_of(unsigned state) {
    int m = 0;
    if (state & ShiftMask) {
        m |= 1;
    }
    if (state & ControlMask) {
        m |= 2;
    }
    if (state & Mod1Mask) {
        m |= 4;
    }
    if (state & Mod4Mask) {
        m |= 8;
    }
    return m;
}

static int code_of(KeySym sym) {
    if (sym == XK_Tab) {
        return 9;
    }
    if (sym == XK_Return || sym == XK_KP_Enter) {
        return 13;
    }
    if (sym == XK_Escape) {
        return 27;
    }
    if (sym == XK_BackSpace) {
        return 8;
    }
    if ((sym & 0xff000000) == 0x01000000) {
        return (int)(sym & 0x00ffffff);
    }
    if (sym >= 0x20 && sym <= 0xFF) {
        return (int)sym;
    }
    return 0;
}

static void want_frame(llr_ui *h) {
    if (h->qn > 0 && h->q[h->qn - 1].kind == LLR_UI_FRAME) {
        return;
    }
    llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
}

static void on_key(llr_ui *h, XKeyEvent *ev, int down) {
    int level = (ev->state & ShiftMask) ? 1 : 0;
    KeySym sym = XkbKeycodeToKeysym(h->dpy, ev->keycode, 0, level);
    int code = code_of(sym);
    char buf[64];
    int n = 0;
    if (code != 0) {
        llr_ui_key(h->q, &h->qn, down, code, mods_of(ev->state));
    }
    if (!down) {
        return;
    }
    if (h->ic) {
        Status st = 0;
        n = Xutf8LookupString(h->ic, ev, buf, (int)sizeof buf - 1, NULL, &st);
    } else {
        n = XLookupString(ev, buf, (int)sizeof buf - 1, NULL, NULL);
    }
    if (n > 0) {
        llr_ui_text(h->q, &h->qn, buf, n);
    }
}

static void on_button(llr_ui *h, XButtonEvent *ev, int down) {
    float s = llr_ui_x11_div(h);
    float x = (float)ev->x / s;
    float y = (float)ev->y / s;
    if (!down) {
        if (ev->button == 1) {
            llr_ui_ptr(h->q, &h->qn, LLR_UI_UP, x, y, 0, 0);
        }
        return;
    }
    if (ev->button == 4) {
        llr_ui_ptr(h->q, &h->qn, LLR_UI_SCROLL, x, y, 0, 1);
        return;
    }
    if (ev->button == 5) {
        llr_ui_ptr(h->q, &h->qn, LLR_UI_SCROLL, x, y, 0, -1);
        return;
    }
    if (ev->button == 6) {
        llr_ui_ptr(h->q, &h->qn, LLR_UI_SCROLL, x, y, 1, 0);
        return;
    }
    if (ev->button == 7) {
        llr_ui_ptr(h->q, &h->qn, LLR_UI_SCROLL, x, y, -1, 0);
        return;
    }
    if (ev->button == 1) {
        llr_ui_ptr(h->q, &h->qn, LLR_UI_DOWN, x, y, 0, 0);
    }
}

static void send_sel(llr_ui *h, XSelectionRequestEvent *req) {
    XSelectionEvent sev;
    memset(&sev, 0, sizeof sev);
    sev.type = SelectionNotify;
    sev.display = req->display;
    sev.requestor = req->requestor;
    sev.selection = req->selection;
    sev.target = req->target;
    sev.time = req->time;
    sev.property = None;
    if (req->target == h->targets) {
        Atom list[2];
        list[0] = h->utf8;
        list[1] = XA_STRING;
        XChangeProperty(h->dpy, req->requestor, req->property, XA_ATOM, 32, PropModeReplace, (unsigned char *)list, 2);
        sev.property = req->property;
    } else if ((req->target == h->utf8 || req->target == XA_STRING) && h->clip) {
        XChangeProperty(h->dpy, req->requestor, req->property, req->target, 8, PropModeReplace, (unsigned char *)h->clip, h->clip_n);
        sev.property = req->property;
    }
    XSendEvent(h->dpy, req->requestor, False, 0, (XEvent *)&sev);
}

void llr_ui_x11_init_input(llr_ui *h) {
    static const unsigned shape[5] = {XC_left_ptr, XC_xterm, XC_hand2, XC_crosshair, XC_watch};
    static int loc = 0;
    int i;
    if (!loc) {
        setlocale(LC_CTYPE, "");
        XSetLocaleModifiers("");
        loc = 1;
    }
    for (i = 0; i < 5; i++) {
        h->cursors[i] = XCreateFontCursor(h->dpy, shape[i]);
    }
    h->im = XOpenIM(h->dpy, NULL, NULL, NULL);
    if (h->im) {
        h->ic = XCreateIC(h->im, XNInputStyle, XIMPreeditNothing | XIMStatusNothing, XNClientWindow, h->win, NULL);
    }
}

void llr_ui_x11_handle(llr_ui *h, XEvent *ev) {
    float s;
    if (!h || !ev) {
        return;
    }
    if (h->ic && XFilterEvent(ev, h->win)) {
        return;
    }
    switch (ev->type) {
    case ButtonPress:
        on_button(h, &ev->xbutton, 1);
        break;
    case ButtonRelease:
        on_button(h, &ev->xbutton, 0);
        break;
    case MotionNotify:
        s = llr_ui_x11_div(h);
        llr_ui_ptr(h->q, &h->qn, LLR_UI_MOVE, (float)ev->xmotion.x / s, (float)ev->xmotion.y / s, 0, 0);
        break;
    case KeyPress:
        on_key(h, &ev->xkey, 1);
        break;
    case KeyRelease:
        on_key(h, &ev->xkey, 0);
        break;
    case ConfigureNotify:
        s = llr_ui_x11_div(h);
        h->logical_w = (int)(ev->xconfigure.width / s + 0.5f);
        h->logical_h = (int)(ev->xconfigure.height / s + 0.5f);
        if (h->logical_w < 1) {
            h->logical_w = 1;
        }
        if (h->logical_h < 1) {
            h->logical_h = 1;
        }
        want_frame(h);
        break;
    case Expose:
        want_frame(h);
        break;
    case ClientMessage:
        if ((Atom)ev->xclient.data.l[0] == h->wm_del) {
            llr_ui_kind(h->q, &h->qn, LLR_UI_QUIT);
        }
        break;
    case SelectionRequest:
        send_sel(h, &ev->xselectionrequest);
        break;
    default:
        break;
    }
}

static int64_t copy_out(const char *src, int n, uint8_t *buf, int64_t cap) {
    if (n < 0) {
        n = 0;
    }
    if (!buf || cap <= 0) {
        return n;
    }
    if ((int64_t)n > cap) {
        n = (int)cap;
    }
    if (n > 0 && src) {
        memcpy(buf, src, (size_t)n);
    }
    return n;
}

int64_t llr_ui_clip_get(void *p, uint8_t *buf, int64_t cap) {
    llr_ui *h = p;
    int fd;
    struct timeval tv;
    if (!h || !h->dpy) {
        return 0;
    }
    if (XGetSelectionOwner(h->dpy, h->clip_atom) == h->win) {
        return copy_out(h->clip, h->clip_n, buf, cap);
    }
    XConvertSelection(h->dpy, h->clip_atom, h->utf8, h->prop, h->win, CurrentTime);
    XFlush(h->dpy);
    fd = ConnectionNumber(h->dpy);
    tv.tv_sec = 0;
    tv.tv_usec = 200000;
    for (;;) {
        while (XPending(h->dpy)) {
            XEvent ev;
            unsigned char *data = NULL;
            unsigned long n = 0;
            Atom actual = 0;
            int format = 0;
            unsigned long left = 0;
            XNextEvent(h->dpy, &ev);
            if (ev.type != SelectionNotify) {
                llr_ui_x11_handle(h, &ev);
                continue;
            }
            if (ev.xselection.property != h->prop) {
                return 0;
            }
            if (XGetWindowProperty(h->dpy, h->win, h->prop, 0, 1 << 20, True, AnyPropertyType, &actual, &format, &n, &left, &data) != Success || !data) {
                return 0;
            }
            n = copy_out((char *)data, (int)n, buf, cap);
            XFree(data);
            return (int64_t)n;
        }
        {
            fd_set fds;
            FD_ZERO(&fds);
            FD_SET(fd, &fds);
            if (select(fd + 1, &fds, NULL, NULL, &tv) <= 0) {
                return 0;
            }
        }
    }
}

void llr_ui_clip_set(void *p, const uint8_t *bytes, int64_t n) {
    llr_ui *h = p;
    if (!h || !h->dpy) {
        return;
    }
    if (n < 0) {
        n = 0;
    }
    free(h->clip);
    h->clip = malloc((size_t)n + 1);
    if (!h->clip) {
        h->clip_n = 0;
        return;
    }
    if (n > 0 && bytes) {
        memcpy(h->clip, bytes, (size_t)n);
    }
    h->clip[n] = 0;
    h->clip_n = (int)n;
    XSetSelectionOwner(h->dpy, h->clip_atom, h->win, CurrentTime);
    XFlush(h->dpy);
}

void llr_ui_set_cursor(void *p, int64_t id) {
    llr_ui *h = p;
    int i = (int)id;
    if (!h || !h->dpy) {
        return;
    }
    if (i < 0 || i > 4) {
        i = 0;
    }
    h->cursor = i;
    if (h->cursors[i]) {
        XDefineCursor(h->dpy, h->win, h->cursors[i]);
    }
}

void llr_ui_text_input(void *p, int64_t on, float x, float y, float w, float hgt) {
    llr_ui *h = p;
    (void)x;
    (void)y;
    (void)w;
    (void)hgt;
    if (!h) {
        return;
    }
    h->text_on = on != 0;
    if (!h->ic) {
        return;
    }
    if (on) {
        XSetICFocus(h->ic);
    } else {
        XUnsetICFocus(h->ic);
    }
}
