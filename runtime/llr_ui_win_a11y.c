/* Win32 UI Automation fragment root. Nodes are a copy of the semantics blob. */
#include "llr_ui_win.h"
#include "llr_ui_a11y.h"

#define COBJMACROS
#include <objbase.h>
#include <initguid.h>
#include <uiautomation.h>
#include <stdlib.h>
#include <string.h>

typedef struct prov prov;

struct prov {
    IRawElementProviderSimpleVtbl *simple;
    IRawElementProviderFragmentVtbl *frag;
    IRawElementProviderFragmentRootVtbl *rootvt;
    IInvokeProviderVtbl *invoke;
    IValueProviderVtbl *value;
    LONG refs;
    llr_ui *host;
    prov *next;
    int root;
    uint32_t id;
    uint32_t role;
    uint32_t flags;
    uint32_t actions;
    float x, y, w, h;
    wchar_t *name;
    wchar_t *val;
};

static prov *from_simple(IRawElementProviderSimple *p) { return (prov *)p; }

static prov *from_frag(IRawElementProviderFragment *p) {
    return (prov *)((char *)p - offsetof(prov, frag));
}

static prov *from_root(IRawElementProviderFragmentRoot *p) {
    return (prov *)((char *)p - offsetof(prov, rootvt));
}

static prov *from_invoke(IInvokeProvider *p) {
    return (prov *)((char *)p - offsetof(prov, invoke));
}

static prov *from_value(IValueProvider *p) {
    return (prov *)((char *)p - offsetof(prov, value));
}

static ULONG add_ref(prov *p) { return (ULONG)InterlockedIncrement(&p->refs); }

static ULONG release(prov *p) {
    ULONG n = (ULONG)InterlockedDecrement(&p->refs);
    if (n == 0) {
        free(p->name);
        free(p->val);
        free(p);
    }
    return n;
}

static CONTROLTYPEID role_type(uint32_t role) {
    switch (role) {
    case 1: return UIA_ButtonControlTypeId;
    case 2: return UIA_CheckBoxControlTypeId;
    case 3: return UIA_ButtonControlTypeId;
    case 4: return UIA_SliderControlTypeId;
    case 5: return UIA_EditControlTypeId;
    case 6: return UIA_HeaderControlTypeId;
    case 7: return UIA_PaneControlTypeId;
    case 8: return UIA_ListItemControlTypeId;
    case 9: return UIA_GroupControlTypeId;
    case 10: return UIA_SeparatorControlTypeId;
    case 11: return UIA_ProgressBarControlTypeId;
    case 12: return UIA_StatusBarControlTypeId;
    case 13: return UIA_TextControlTypeId;
    case 14: return UIA_ImageControlTypeId;
    default: return UIA_GroupControlTypeId;
    }
}

static wchar_t *widen(const char *s) {
    int n;
    wchar_t *w;
    if (!s) {
        s = "";
    }
    n = MultiByteToWideChar(CP_UTF8, 0, s, -1, NULL, 0);
    if (n < 1) {
        n = 1;
    }
    w = (wchar_t *)calloc((size_t)n, sizeof(wchar_t));
    if (w) {
        MultiByteToWideChar(CP_UTF8, 0, s, -1, w, n);
    }
    return w;
}

static int node_at(const llr_a11y_tree *t, uint32_t id) {
    uint32_t i;
    if (!t) {
        return -1;
    }
    for (i = 0; i < t->count; i++) {
        if (t->nodes[i].id == id) {
            return (int)i;
        }
    }
    return -1;
}

static int child_n(const llr_a11y_tree *t, uint32_t parent) {
    uint32_t i;
    int n = 0;
    if (!t) {
        return 0;
    }
    for (i = 0; i < t->count; i++) {
        if (t->nodes[i].parent == parent) {
            n++;
        }
    }
    return n;
}

static int child_i(const llr_a11y_tree *t, uint32_t parent, int nth) {
    uint32_t i;
    int seen = 0;
    for (i = 0; t && i < t->count; i++) {
        if (t->nodes[i].parent == parent) {
            if (seen == nth) {
                return (int)i;
            }
            seen++;
        }
    }
    return -1;
}

static HRESULT fill_node(prov *p, const llr_a11y_node *n) {
    p->id = n->id;
    p->role = n->role;
    p->flags = n->flags;
    p->actions = n->actions;
    p->x = n->x;
    p->y = n->y;
    p->w = n->w;
    p->h = n->h;
    p->name = widen(n->label && n->label[0] ? n->label : n->value);
    p->val = widen(n->value);
    if (!p->name || !p->val) {
        return E_OUTOFMEMORY;
    }
    return S_OK;
}

static prov *new_prov(llr_ui *h, int root);
static HRESULT prov_qi(prov *p, REFIID iid, void **out);

static IRawElementProviderSimple *as_simple(prov *p) {
    return p ? (IRawElementProviderSimple *)p : NULL;
}

static IRawElementProviderFragment *as_frag(prov *p) {
    return p ? (IRawElementProviderFragment *)&p->frag : NULL;
}

static prov *make_id(llr_ui *h, uint32_t id) {
    prov *p;
    int at;
    if (!h || !h->acc) {
        return NULL;
    }
    at = node_at((llr_a11y_tree *)h->acc, id);
    if (at < 0) {
        return NULL;
    }
    p = new_prov(h, 0);
    if (!p) {
        return NULL;
    }
    if (fill_node(p, &((llr_a11y_tree *)h->acc)->nodes[at]) != S_OK) {
        release(p);
        return NULL;
    }
    return p;
}

static HRESULT STDMETHODCALLTYPE s_qi(IRawElementProviderSimple *self, REFIID iid, void **out) {
    return prov_qi(from_simple(self), iid, out);
}
static ULONG STDMETHODCALLTYPE s_add(IRawElementProviderSimple *self) { return add_ref(from_simple(self)); }
static ULONG STDMETHODCALLTYPE s_rel(IRawElementProviderSimple *self) { return release(from_simple(self)); }
static HRESULT STDMETHODCALLTYPE f_qi(IRawElementProviderFragment *self, REFIID iid, void **out) {
    return prov_qi(from_frag(self), iid, out);
}
static ULONG STDMETHODCALLTYPE f_add(IRawElementProviderFragment *self) { return add_ref(from_frag(self)); }
static ULONG STDMETHODCALLTYPE f_rel(IRawElementProviderFragment *self) { return release(from_frag(self)); }
static HRESULT STDMETHODCALLTYPE r_qi(IRawElementProviderFragmentRoot *self, REFIID iid, void **out) {
    return prov_qi(from_root(self), iid, out);
}
static ULONG STDMETHODCALLTYPE r_add(IRawElementProviderFragmentRoot *self) { return add_ref(from_root(self)); }
static ULONG STDMETHODCALLTYPE r_rel(IRawElementProviderFragmentRoot *self) { return release(from_root(self)); }
static HRESULT STDMETHODCALLTYPE i_qi(IInvokeProvider *self, REFIID iid, void **out) {
    return prov_qi(from_invoke(self), iid, out);
}
static ULONG STDMETHODCALLTYPE i_add(IInvokeProvider *self) { return add_ref(from_invoke(self)); }
static ULONG STDMETHODCALLTYPE i_rel(IInvokeProvider *self) { return release(from_invoke(self)); }
static HRESULT STDMETHODCALLTYPE v_qi(IValueProvider *self, REFIID iid, void **out) {
    return prov_qi(from_value(self), iid, out);
}
static ULONG STDMETHODCALLTYPE v_add(IValueProvider *self) { return add_ref(from_value(self)); }
static ULONG STDMETHODCALLTYPE v_rel(IValueProvider *self) { return release(from_value(self)); }

static HRESULT STDMETHODCALLTYPE s_opt(IRawElementProviderSimple *self, enum ProviderOptions *opt) {
    (void)self;
    *opt = ProviderOptions_ServerSideProvider;
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE s_pat(IRawElementProviderSimple *self, PATTERNID id, IUnknown **out) {
    prov *p = from_simple(self);
    *out = NULL;
    if (id == UIA_InvokePatternId && (p->actions & 1)) {
        *out = (IUnknown *)&p->invoke;
        add_ref(p);
    } else if (id == UIA_ValuePatternId && (p->actions & 4)) {
        *out = (IUnknown *)&p->value;
        add_ref(p);
    }
    return S_OK;
}

static HRESULT put_rect(prov *p, VARIANT *v) {
    SAFEARRAY *sa;
    double *d;
    POINT pt;
    float sc;
    sa = SafeArrayCreateVector(VT_R8, 0, 4);
    if (!sa) {
        return E_OUTOFMEMORY;
    }
    pt.x = 0;
    pt.y = 0;
    sc = 1.0f;
    if (p->host && p->host->hwnd) {
        ClientToScreen(p->host->hwnd, &pt);
        sc = p->host->scale < 0.01f ? 1.0f : p->host->scale;
    }
    SafeArrayAccessData(sa, (void **)&d);
    d[0] = pt.x + (double)p->x * sc;
    d[1] = pt.y + (double)p->y * sc;
    d[2] = (double)p->w * sc;
    d[3] = (double)p->h * sc;
    SafeArrayUnaccessData(sa);
    v->vt = VT_ARRAY | VT_R8;
    v->parray = sa;
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE s_prop(IRawElementProviderSimple *self, PROPERTYID id, VARIANT *v) {
    prov *p = from_simple(self);
    VariantInit(v);
    if (id == UIA_NamePropertyId) {
        v->vt = VT_BSTR;
        v->bstrVal = SysAllocString(p->name ? p->name : L"");
        return v->bstrVal ? S_OK : E_OUTOFMEMORY;
    }
    if (id == UIA_ControlTypePropertyId) {
        v->vt = VT_I4;
        v->lVal = p->root ? UIA_WindowControlTypeId : (LONG)role_type(p->role);
        return S_OK;
    }
    if (id == UIA_IsKeyboardFocusablePropertyId || id == UIA_IsEnabledPropertyId) {
        v->vt = VT_BOOL;
        v->boolVal = (p->flags & 1) || p->root ? VARIANT_TRUE : VARIANT_FALSE;
        if (id == UIA_IsEnabledPropertyId) {
            v->boolVal = VARIANT_TRUE;
        }
        return S_OK;
    }
    if (id == UIA_BoundingRectanglePropertyId) {
        return put_rect(p, v);
    }
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE s_host(IRawElementProviderSimple *self, IRawElementProviderSimple **out) {
    prov *p = from_simple(self);
    *out = NULL;
    if (p->root && p->host && p->host->hwnd) {
        return UiaHostProviderFromHwnd(p->host->hwnd, out);
    }
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE f_nav(IRawElementProviderFragment *self, enum NavigateDirection dir, IRawElementProviderFragment **out) {
    prov *p = from_frag(self);
    llr_a11y_tree *t;
    uint32_t parent;
    int at;
    int nth;
    *out = NULL;
    if (!p->host || !p->host->acc) {
        return S_OK;
    }
    t = (llr_a11y_tree *)p->host->acc;
    if (p->root) {
        if (dir == NavigateDirection_FirstChild) {
            at = child_i(t, 0xFFFFFFFFu, 0);
        } else if (dir == NavigateDirection_LastChild) {
            at = child_i(t, 0xFFFFFFFFu, child_n(t, 0xFFFFFFFFu) - 1);
        } else {
            at = -1;
        }
        if (at >= 0) {
            *out = as_frag(make_id(p->host, t->nodes[at].id));
        }
        return S_OK;
    }
    at = node_at(t, p->id);
    parent = at < 0 ? 0xFFFFFFFFu : t->nodes[at].parent;
    if (dir == NavigateDirection_Parent) {
        if (parent == 0xFFFFFFFFu) {
            *out = as_frag((prov *)p->host->acc_root);
            if (*out) {
                add_ref((prov *)p->host->acc_root);
            }
        } else {
            *out = as_frag(make_id(p->host, parent));
        }
        return S_OK;
    }
    nth = 0;
    if (dir == NavigateDirection_NextSibling || dir == NavigateDirection_PreviousSibling) {
        int i;
        int cur = -1;
        int n = child_n(t, parent);
        for (i = 0; i < n; i++) {
            int ix = child_i(t, parent, i);
            if (ix >= 0 && t->nodes[ix].id == p->id) {
                cur = i;
            }
        }
        nth = dir == NavigateDirection_NextSibling ? cur + 1 : cur - 1;
        at = nth < 0 ? -1 : child_i(t, parent, nth);
    } else if (dir == NavigateDirection_FirstChild) {
        at = child_i(t, p->id, 0);
    } else if (dir == NavigateDirection_LastChild) {
        at = child_i(t, p->id, child_n(t, p->id) - 1);
    } else {
        at = -1;
    }
    if (at >= 0) {
        *out = as_frag(make_id(p->host, t->nodes[at].id));
    }
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE f_rid(IRawElementProviderFragment *self, SAFEARRAY **out) {
    prov *p = from_frag(self);
    int *d;
    SAFEARRAY *sa = SafeArrayCreateVector(VT_I4, 0, 2);
    if (!sa) {
        return E_OUTOFMEMORY;
    }
    SafeArrayAccessData(sa, (void **)&d);
    d[0] = UiaAppendRuntimeId;
    d[1] = p->root ? 1 : (int)p->id + 2;
    SafeArrayUnaccessData(sa);
    *out = sa;
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE f_box(IRawElementProviderFragment *self, struct UiaRect *r) {
    VARIANT v;
    prov *p = from_frag(self);
    double *d;
    HRESULT hr = put_rect(p, &v);
    if (FAILED(hr)) {
        return hr;
    }
    SafeArrayAccessData(v.parray, (void **)&d);
    r->left = d[0];
    r->top = d[1];
    r->width = d[2];
    r->height = d[3];
    SafeArrayUnaccessData(v.parray);
    VariantClear(&v);
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE f_embed(IRawElementProviderFragment *self, SAFEARRAY **out) {
    (void)self;
    *out = NULL;
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE f_focus(IRawElementProviderFragment *self) {
    prov *p = from_frag(self);
    if (p->host) {
        llr_ui_action(p->host->q, &p->host->qn, 2, (int)p->id, NULL, 0);
    }
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE f_root(IRawElementProviderFragment *self, IRawElementProviderFragmentRoot **out) {
    prov *p = from_frag(self);
    *out = NULL;
    if (p->host && p->host->acc_root) {
        prov *root = (prov *)p->host->acc_root;
        *out = (IRawElementProviderFragmentRoot *)&root->rootvt;
        add_ref(root);
    }
    return S_OK;
}

static int hit_id(llr_ui *h, double x, double y) {
    llr_a11y_tree *t;
    POINT pt;
    float sc;
    int best = -1;
    uint32_t i;
    if (!h || !h->hwnd || !h->acc) {
        return -1;
    }
    pt.x = (LONG)x;
    pt.y = (LONG)y;
    ScreenToClient(h->hwnd, &pt);
    sc = h->scale < 0.01f ? 1.0f : h->scale;
    t = (llr_a11y_tree *)h->acc;
    for (i = 0; i < t->count; i++) {
        double nx = pt.x / sc;
        double ny = pt.y / sc;
        llr_a11y_node *n = &t->nodes[i];
        if (nx >= n->x && ny >= n->y && nx < n->x + n->w && ny < n->y + n->h) {
            best = (int)i;
        }
    }
    return best;
}

static HRESULT STDMETHODCALLTYPE r_at(IRawElementProviderFragmentRoot *self, double x, double y, IRawElementProviderFragment **out) {
    prov *p = from_root(self);
    int at = hit_id(p->host, x, y);
    *out = NULL;
    if (at >= 0 && p->host && p->host->acc) {
        *out = as_frag(make_id(p->host, ((llr_a11y_tree *)p->host->acc)->nodes[at].id));
    }
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE r_get(IRawElementProviderFragmentRoot *self, IRawElementProviderFragment **out) {
    (void)self;
    *out = NULL;
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE i_do(IInvokeProvider *self) {
    prov *p = from_invoke(self);
    int bit = (p->actions & 8) ? 8 : 1;
    if (p->host) {
        llr_ui_action(p->host->q, &p->host->qn, bit, (int)p->id, NULL, 0);
    }
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE v_set(IValueProvider *self, LPCWSTR text) {
    prov *p = from_value(self);
    char buf[32];
    int n;
    if (!p->host) {
        return S_OK;
    }
    n = WideCharToMultiByte(CP_UTF8, 0, text ? text : L"", -1, buf, 32, NULL, NULL);
    if (n < 1) {
        n = 1;
    }
    llr_ui_action(p->host->q, &p->host->qn, 4, (int)p->id, buf, n - 1);
    return S_OK;
}

static HRESULT STDMETHODCALLTYPE v_get(IValueProvider *self, BSTR *out) {
    prov *p = from_value(self);
    *out = SysAllocString(p->val ? p->val : L"");
    return *out ? S_OK : E_OUTOFMEMORY;
}

static HRESULT STDMETHODCALLTYPE v_ro(IValueProvider *self, BOOL *out) {
    (void)self;
    *out = FALSE;
    return S_OK;
}

static IRawElementProviderSimpleVtbl g_simple = {s_qi, s_add, s_rel, s_opt, s_pat, s_prop, s_host};
static IRawElementProviderFragmentVtbl g_frag = {f_qi, f_add, f_rel, f_nav, f_rid, f_box, f_embed, f_focus, f_root};
static IRawElementProviderFragmentRootVtbl g_root = {r_qi, r_add, r_rel, r_at, r_get};
static IInvokeProviderVtbl g_invoke = {i_qi, i_add, i_rel, i_do};
static IValueProviderVtbl g_value = {v_qi, v_add, v_rel, v_set, v_get, v_ro};

static HRESULT prov_qi(prov *p, REFIID iid, void **out) {
    *out = NULL;
    if (IsEqualIID(iid, &IID_IUnknown) || IsEqualIID(iid, &IID_IRawElementProviderSimple)) {
        *out = p;
    } else if (IsEqualIID(iid, &IID_IRawElementProviderFragment)) {
        *out = &p->frag;
    } else if (p->root && IsEqualIID(iid, &IID_IRawElementProviderFragmentRoot)) {
        *out = &p->rootvt;
    } else if (IsEqualIID(iid, &IID_IInvokeProvider) && (p->actions & 1)) {
        *out = &p->invoke;
    } else if (IsEqualIID(iid, &IID_IValueProvider) && (p->actions & 4)) {
        *out = &p->value;
    } else {
        return E_NOINTERFACE;
    }
    add_ref(p);
    return S_OK;
}

static prov *new_prov(llr_ui *h, int root) {
    prov *p = (prov *)calloc(1, sizeof *p);
    if (!p) {
        return NULL;
    }
    p->simple = &g_simple;
    p->frag = &g_frag;
    p->rootvt = &g_root;
    p->invoke = &g_invoke;
    p->value = &g_value;
    p->refs = 1;
    p->host = h;
    p->root = root;
    p->name = widen("");
    p->val = widen("");
    if (h) {
        p->next = (prov *)h->acc_list;
        h->acc_list = p;
    }
    return p;
}

static void drop_tree(llr_ui *h) {
    prov *p;
    if (h->acc) {
        llr_a11y_clear((llr_a11y_tree *)h->acc);
        free(h->acc);
        h->acc = NULL;
    }
    for (p = (prov *)h->acc_list; p; p = p->next) {
        p->host = NULL;
    }
    h->acc_list = NULL;
    if (h->acc_root) {
        release((prov *)h->acc_root);
        h->acc_root = NULL;
    }
}

void llr_ui_win_a11y_free(llr_ui *h) {
    if (h) {
        drop_tree(h);
    }
}

LRESULT llr_ui_win_a11y_get(llr_ui *h, HWND hw, WPARAM wp, LPARAM lp) {
    if (!h || !h->acc_root || (LONG)lp != (LONG)UiaRootObjectId) {
        return 0;
    }
    return UiaReturnRawElementProvider(hw, wp, lp, as_simple((prov *)h->acc_root));
}

void llr_ui_a11y_update(void *raw, const uint8_t *bytes, int64_t n) {
    llr_ui *h = (llr_ui *)raw;
    llr_a11y_tree *t;
    prov *root;
    if (!h) {
        return;
    }
    CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);
    drop_tree(h);
    t = (llr_a11y_tree *)calloc(1, sizeof *t);
    if (!t) {
        return;
    }
    if (!llr_a11y_parse(t, bytes, n)) {
        free(t);
        return;
    }
    h->acc = t;
    root = new_prov(h, 1);
    h->acc_root = root;
    if (h->hwnd) {
        UiaRaiseAutomationEvent(as_simple(root), UIA_StructureChangedEventId);
    }
}
