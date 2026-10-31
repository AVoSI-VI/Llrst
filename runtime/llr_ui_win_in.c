/* Win32 pointer, key, committed IME text, clipboard, and cursor. */
#include "llr_ui_win.h"

#include <imm.h>
#include <stdlib.h>

static int mods(void) {
    int m = 0;
    if (GetKeyState(VK_SHIFT) & 0x8000) {
        m |= 1;
    }
    if (GetKeyState(VK_CONTROL) & 0x8000) {
        m |= 2;
    }
    if (GetKeyState(VK_MENU) & 0x8000) {
        m |= 4;
    }
    if ((GetKeyState(VK_LWIN) & 0x8000) || (GetKeyState(VK_RWIN) & 0x8000)) {
        m |= 8;
    }
    return m;
}

static int key_code(WPARAM vk) {
    UINT ch;
    switch (vk) {
    case VK_TAB:
        return 9;
    case VK_RETURN:
        return 13;
    case VK_ESCAPE:
        return 27;
    case VK_BACK:
        return 8;
    case VK_SHIFT:
    case VK_CONTROL:
    case VK_MENU:
    case VK_LWIN:
    case VK_RWIN:
        return 0;
    default:
        break;
    }
    ch = MapVirtualKeyW((UINT)vk, MAPVK_VK_TO_CHAR) & 0xFFFF;
    if (ch >= 32 && ch < 0xD800) {
        return (int)ch;
    }
    return (int)vk;
}

static void at_client(llr_ui *h, LPARAM lp, float *x, float *y) {
    float s = llr_ui_win_div(h);
    *x = (float)(short)LOWORD(lp) / s;
    *y = (float)(short)HIWORD(lp) / s;
}

static int utf16_utf8(const uint16_t *in, int n, uint8_t *out, int cap) {
    int o = 0;
    int i;
    for (i = 0; i < n; i++) {
        uint32_t cp = in[i];
        uint8_t tmp[4];
        int k;
        if (cp >= 0xD800 && cp <= 0xDBFF && i + 1 < n) {
            uint32_t lo = in[i + 1];
            if (lo >= 0xDC00 && lo <= 0xDFFF) {
                cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                i++;
            }
        }
        k = llr_ui_utf8(cp, tmp);
        if (k == 0) {
            continue;
        }
        if (out && o + k <= cap) {
            memcpy(out + o, tmp, (size_t)k);
        }
        o += k;
    }
    return o;
}

static void push_utf16(llr_ui *h, const uint16_t *in, int n) {
    int len = utf16_utf8(in, n, NULL, 0);
    uint8_t *buf;
    if (len <= 0) {
        return;
    }
    buf = malloc((size_t)len);
    if (!buf) {
        return;
    }
    utf16_utf8(in, n, buf, len);
    llr_ui_text(h->q, &h->qn, buf, len);
    free(buf);
}

static void on_char(llr_ui *h, WPARAM wp) {
    uint16_t c = (uint16_t)wp;
    uint16_t pair[2];
    int n = 1;
    if (c < 32) {
        return;
    }
    pair[0] = c;
    if (c >= 0xD800 && c <= 0xDBFF) {
        h->high = c;
        return;
    }
    if (h->high && c >= 0xDC00 && c <= 0xDFFF) {
        pair[0] = h->high;
        pair[1] = c;
        n = 2;
    }
    h->high = 0;
    push_utf16(h, pair, n);
}

static void ime_result(llr_ui *h) {
    HIMC imc = ImmGetContext(h->hwnd);
    LONG bytes;
    uint16_t *buf;
    if (!imc) {
        return;
    }
    bytes = ImmGetCompositionStringW(imc, GCS_RESULTSTR, NULL, 0);
    if (bytes > 0) {
        buf = malloc((size_t)bytes + sizeof(uint16_t));
        if (buf) {
            ImmGetCompositionStringW(imc, GCS_RESULTSTR, buf, bytes);
            push_utf16(h, buf, (int)(bytes / 2));
            free(buf);
        }
    }
    ImmReleaseContext(h->hwnd, imc);
}

static void ime_place(llr_ui *h) {
    HIMC imc;
    COMPOSITIONFORM cf;
    if (!h->hwnd || !h->text_on) {
        return;
    }
    imc = ImmGetContext(h->hwnd);
    if (!imc) {
        return;
    }
    cf.dwStyle = CFS_POINT;
    cf.ptCurrentPos.x = (LONG)(h->tx * h->scale);
    cf.ptCurrentPos.y = (LONG)((h->ty + h->th) * h->scale);
    ImmSetCompositionWindow(imc, &cf);
    ImmReleaseContext(h->hwnd, imc);
}

static HCURSOR cur_of(int id) {
    LPCWSTR name = IDC_ARROW;
    switch (id) {
    case LLR_UI_CUR_TEXT:
        name = IDC_IBEAM;
        break;
    case LLR_UI_CUR_HAND:
        name = IDC_HAND;
        break;
    case LLR_UI_CUR_CROSS:
        name = IDC_CROSS;
        break;
    case LLR_UI_CUR_WAIT:
        name = IDC_WAIT;
        break;
    default:
        break;
    }
    return LoadCursorW(NULL, name);
}

static int on_pointer(llr_ui *h, UINT msg, WPARAM wp, LPARAM lp) {
    float x;
    float y;
    POINT pt;
    if (msg == WM_LBUTTONDOWN) {
        SetCapture(h->hwnd);
        at_client(h, lp, &x, &y);
        llr_ui_ptr(h->q, &h->qn, LLR_UI_DOWN, x, y, 0, 0);
        return 1;
    }
    if (msg == WM_LBUTTONUP) {
        ReleaseCapture();
        at_client(h, lp, &x, &y);
        llr_ui_ptr(h->q, &h->qn, LLR_UI_UP, x, y, 0, 0);
        return 1;
    }
    if (msg == WM_MOUSEMOVE) {
        at_client(h, lp, &x, &y);
        llr_ui_ptr(h->q, &h->qn, LLR_UI_MOVE, x, y, 0, 0);
        return 1;
    }
    if (msg != WM_MOUSEWHEEL) {
        return 0;
    }
    pt.x = (short)LOWORD(lp);
    pt.y = (short)HIWORD(lp);
    ScreenToClient(h->hwnd, &pt);
    x = (float)pt.x / llr_ui_win_div(h);
    y = (float)pt.y / llr_ui_win_div(h);
    if (LOWORD(wp) & MK_SHIFT) {
        llr_ui_ptr(h->q, &h->qn, LLR_UI_SCROLL, x, y, (float)(short)HIWORD(wp) / 120.0f, 0);
    } else {
        llr_ui_ptr(h->q, &h->qn, LLR_UI_SCROLL, x, y, 0, (float)(short)HIWORD(wp) / 120.0f);
    }
    return 1;
}

int llr_ui_win_input(llr_ui *h, UINT msg, WPARAM wp, LPARAM lp) {
    int code;
    if (!h) {
        return 0;
    }
    if (on_pointer(h, msg, wp, lp)) {
        return 1;
    }
    if (msg == WM_KEYDOWN || msg == WM_KEYUP || msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP) {
        if (wp == VK_PROCESSKEY) {
            return 1;
        }
        code = key_code(wp);
        if (code != 0) {
            int down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            llr_ui_key(h->q, &h->qn, down, code, mods());
        }
        return 0;
    }
    if (msg == WM_CHAR) {
        on_char(h, wp);
        return 1;
    }
    if (msg == WM_IME_COMPOSITION && (lp & GCS_RESULTSTR)) {
        ime_result(h);
        return 0;
    }
    if (msg == WM_SETCURSOR && LOWORD(lp) == HTCLIENT) {
        SetCursor(cur_of(h->cursor));
        return 1;
    }
    return 0;
}

void llr_ui_set_cursor(void *p, int64_t id) {
    llr_ui *h = p;
    if (!h) {
        return;
    }
    h->cursor = (int)id;
    SetCursor(cur_of(h->cursor));
}

void llr_ui_text_input(void *p, int64_t on, float x, float y, float w, float hgt) {
    llr_ui *h = p;
    if (!h) {
        return;
    }
    h->text_on = on != 0;
    h->tx = x;
    h->ty = y;
    h->tw = w;
    h->th = hgt;
    ime_place(h);
}

int64_t llr_ui_clip_get(void *p, uint8_t *buf, int64_t cap) {
    llr_ui *h = p;
    HANDLE mem;
    wchar_t *w;
    int chars;
    int bytes;
    if (!h || !OpenClipboard(h->hwnd)) {
        return 0;
    }
    mem = GetClipboardData(CF_UNICODETEXT);
    if (!mem) {
        CloseClipboard();
        return 0;
    }
    w = GlobalLock(mem);
    if (!w) {
        CloseClipboard();
        return 0;
    }
    chars = (int)wcslen(w);
    bytes = WideCharToMultiByte(CP_UTF8, 0, w, chars, NULL, 0, NULL, NULL);
    if (buf && cap > 0 && bytes > 0) {
        int n = bytes > cap ? (int)cap : bytes;
        WideCharToMultiByte(CP_UTF8, 0, w, chars, (char *)buf, n, NULL, NULL);
        if (bytes > cap) {
            bytes = (int)cap;
        }
    }
    GlobalUnlock(mem);
    CloseClipboard();
    return bytes;
}

void llr_ui_clip_set(void *p, const uint8_t *bytes, int64_t n) {
    llr_ui *h = p;
    int chars;
    HGLOBAL mem;
    wchar_t *w;
    if (!h || !h->hwnd) {
        return;
    }
    if (n < 0) {
        n = 0;
    }
    chars = MultiByteToWideChar(CP_UTF8, 0, (const char *)bytes, (int)n, NULL, 0);
    mem = GlobalAlloc(GMEM_MOVEABLE, (SIZE_T)(chars + 1) * sizeof(wchar_t));
    if (!mem) {
        return;
    }
    w = GlobalLock(mem);
    if (!w) {
        GlobalFree(mem);
        return;
    }
    MultiByteToWideChar(CP_UTF8, 0, (const char *)bytes, (int)n, w, chars);
    w[chars] = 0;
    GlobalUnlock(mem);
    if (!OpenClipboard(h->hwnd)) {
        GlobalFree(mem);
        return;
    }
    EmptyClipboard();
    if (!SetClipboardData(CF_UNICODETEXT, mem)) {
        GlobalFree(mem);
    }
    CloseClipboard();
}
