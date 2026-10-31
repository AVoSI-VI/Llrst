/* Win32 surface. The process is per-monitor DPI aware. Present is a top-down BGRA blit. */
#define WIN32_LEAN_AND_MEAN
#include "llr_ui_win.h"

#include <stdlib.h>

float llr_ui_win_div(llr_ui *h) {
    if (!h || h->scale < 0.01f) {
        return 1.0f;
    }
    return h->scale;
}

static void apply_client(llr_ui *h) {
    RECT rc;
    float s = llr_ui_win_div(h);
    int w;
    int ht;
    if (!h->hwnd) {
        return;
    }
    GetClientRect(h->hwnd, &rc);
    w = (int)((rc.right - rc.left) / s + 0.5f);
    ht = (int)((rc.bottom - rc.top) / s + 0.5f);
    if (w < 1) {
        w = 1;
    }
    if (ht < 1) {
        ht = 1;
    }
    h->logical_w = w;
    h->logical_h = ht;
    llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
}

static void fit_client(llr_ui *h) {
    UINT dpi = (UINT)(h->scale * 96.0f + 0.5f);
    RECT rc;
    int cw = (int)(h->logical_w * h->scale + 0.5f);
    int ch = (int)(h->logical_h * h->scale + 0.5f);
    if (dpi == 0) {
        dpi = 96;
    }
    if (cw < 1) {
        cw = 1;
    }
    if (ch < 1) {
        ch = 1;
    }
    rc.left = 0;
    rc.top = 0;
    rc.right = cw;
    rc.bottom = ch;
    AdjustWindowRectExForDpi(&rc, WS_OVERLAPPEDWINDOW | WS_VISIBLE, FALSE, 0, dpi);
    SetWindowPos(h->hwnd, NULL, 0, 0, rc.right - rc.left, rc.bottom - rc.top, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
}

static LRESULT CALLBACK wndproc(HWND hw, UINT msg, WPARAM wp, LPARAM lp) {
    llr_ui *h;
    if (msg == WM_NCCREATE) {
        CREATESTRUCTW *cs = (CREATESTRUCTW *)lp;
        SetWindowLongPtrW(hw, GWLP_USERDATA, (LONG_PTR)cs->lpCreateParams);
    }
    h = (llr_ui *)GetWindowLongPtrW(hw, GWLP_USERDATA);
    if (!h) {
        return DefWindowProcW(hw, msg, wp, lp);
    }
    if (msg == WM_ERASEBKGND) {
        return 1;
    }
    if (msg == WM_CLOSE) {
        llr_ui_kind(h->q, &h->qn, LLR_UI_QUIT);
        return 0;
    }
    if (msg == WM_SIZE && h->ready) {
        apply_client(h);
        return 0;
    }
    if (msg == WM_DPICHANGED) {
        RECT *r = (RECT *)lp;
        h->scale = (float)HIWORD(wp) / 96.0f;
        if (h->scale < 0.01f) {
            h->scale = 1.0f;
        }
        SetWindowPos(hw, NULL, r->left, r->top, r->right - r->left, r->bottom - r->top, SWP_NOZORDER | SWP_NOACTIVATE);
        return 0;
    }
    if (msg == WM_PAINT) {
        PAINTSTRUCT ps;
        BeginPaint(hw, &ps);
        EndPaint(hw, &ps);
        llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
        return 0;
    }
    if (llr_ui_win_input(h, msg, wp, lp)) {
        return msg == WM_SETCURSOR ? TRUE : 0;
    }
    return DefWindowProcW(hw, msg, wp, lp);
}

static void dpi_once(void) {
    static int done = 0;
    if (done) {
        return;
    }
    done = 1;
    SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
}

static void class_once(void) {
    static int done = 0;
    WNDCLASSW wc;
    if (done) {
        return;
    }
    done = 1;
    memset(&wc, 0, sizeof wc);
    wc.lpfnWndProc = wndproc;
    wc.hInstance = GetModuleHandleW(NULL);
    wc.lpszClassName = L"llr_ui";
    wc.hCursor = LoadCursorW(NULL, IDC_ARROW);
    RegisterClassW(&wc);
}

static void pump(llr_ui *h, int block) {
    MSG msg;
    if (block) {
        if (GetMessageW(&msg, NULL, 0, 0) <= 0) {
            llr_ui_kind(h->q, &h->qn, LLR_UI_QUIT);
            return;
        }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
        if (msg.message == WM_QUIT) {
            llr_ui_kind(h->q, &h->qn, LLR_UI_QUIT);
            return;
        }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}

static void destroy(llr_ui *h) {
    HWND hw;
    if (!h) {
        return;
    }
    hw = h->hwnd;
    h->hwnd = NULL;
    if (hw) {
        SetWindowLongPtrW(hw, GWLP_USERDATA, 0);
        DestroyWindow(hw);
    }
    free(h);
}

void *llr_ui_open(int64_t w, int64_t hgt) {
    llr_ui *h = calloc(1, sizeof *h);
    UINT dpi;
    if (!h) {
        return NULL;
    }
    if (w < 1) {
        w = 1;
    }
    if (hgt < 1) {
        hgt = 1;
    }
    h->logical_w = (int)w;
    h->logical_h = (int)hgt;
    h->scale = 1.0f;
    h->first = 1;
    dpi_once();
    class_once();
    h->hwnd = CreateWindowExW(0, L"llr_ui", L"llr", WS_OVERLAPPEDWINDOW | WS_VISIBLE, CW_USEDEFAULT, CW_USEDEFAULT, (int)w, (int)hgt, NULL, NULL, GetModuleHandleW(NULL), h);
    if (!h->hwnd) {
        free(h);
        return NULL;
    }
    dpi = GetDpiForWindow(h->hwnd);
    h->scale = dpi == 0 ? 1.0f : (float)dpi / 96.0f;
    fit_client(h);
    h->ready = 1;
    apply_client(h);
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
    return h ? llr_ui_win_div(h) : 1.0f;
}

void llr_ui_present(void *p, const uint8_t *px, int64_t w, int64_t hgt, int64_t stride) {
    llr_ui *h = p;
    BITMAPINFO bi;
    HDC dc;
    RECT rc;
    int cw;
    int ch;
    (void)stride;
    if (!h || !h->hwnd || !px || w <= 0 || hgt <= 0) {
        return;
    }
    GetClientRect(h->hwnd, &rc);
    cw = rc.right - rc.left;
    ch = rc.bottom - rc.top;
    if (cw < 1) {
        cw = 1;
    }
    if (ch < 1) {
        ch = 1;
    }
    memset(&bi, 0, sizeof bi);
    bi.bmiHeader.biSize = sizeof bi.bmiHeader;
    bi.bmiHeader.biWidth = (LONG)w;
    bi.bmiHeader.biHeight = -(LONG)hgt;
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;
    dc = GetDC(h->hwnd);
    if (!dc) {
        return;
    }
    StretchDIBits(dc, 0, 0, cw, ch, 0, 0, (int)w, (int)hgt, px, &bi, DIB_RGB_COLORS, SRCCOPY);
    ReleaseDC(h->hwnd, dc);
}

int64_t llr_ui_poll(void *p, uint8_t *buf, int64_t cap) {
    llr_ui *h = p;
    if (!h) {
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
        pump(h, 0);
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
    (void)p;
    return (int64_t)GetTickCount64();
}
