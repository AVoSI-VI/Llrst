/* X11 host state shared by the window file and the input file. */
#ifndef LLR_UI_X11_H
#define LLR_UI_X11_H

#define _POSIX_C_SOURCE 200809L
#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/keysym.h>

#include "llr_ui.h"

typedef struct llr_ui llr_ui;

struct llr_ui {
    Display *dpy;
    int screen;
    Window win;
    GC gc;
    Atom wm_del;
    Atom clip_atom;
    Atom utf8;
    Atom targets;
    Atom prop;
    XIM im;
    XIC ic;
    int logical_w;
    int logical_h;
    float scale;
    int cursor;
    int frame;
    int first;
    int text_on;
    int direct;
    int msb;
    int rs, gs, bs;
    Cursor cursors[5];
    char *clip;
    int clip_n;
    uint8_t *sw;
    size_t sw_n;
    llr_ui_ev q[LLR_UI_RING];
    int qn;
};

void llr_ui_x11_handle(llr_ui *h, XEvent *ev);
void llr_ui_x11_init_input(llr_ui *h);
float llr_ui_x11_div(llr_ui *h);

#endif
