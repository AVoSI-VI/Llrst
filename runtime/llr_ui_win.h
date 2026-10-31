/* Win32 host state shared by the window file and the input file. */
#ifndef LLR_UI_WIN_H
#define LLR_UI_WIN_H

#define WIN32_LEAN_AND_MEAN
#define _WIN32_WINNT 0x0A00
#include <windows.h>

#include "llr_ui.h"

typedef struct llr_ui {
    HWND hwnd;
    int logical_w;
    int logical_h;
    float scale;
    int cursor;
    int frame;
    int first;
    int ready;
    int text_on;
    float tx, ty, tw, th;
    uint16_t high;
    llr_ui_ev q[LLR_UI_RING];
    int qn;
} llr_ui;

void llr_ui_win_input(llr_ui *h, UINT msg, WPARAM wp, LPARAM lp);
float llr_ui_win_div(llr_ui *h);

#endif
