/* Android input, lifecycle, insets, clipboard, and the soft keyboard. */
#include "llr_ui_android.h"

#include <android/configuration.h>
#include <android/input.h>
#include <android/native_window.h>
#include <jni.h>
#include <stdlib.h>
#include <string.h>

float llr_and_density(llr_and *h) {
    int dpi;
    if (!h || !h->app || !h->app->config) {
        return 1.0f;
    }
    dpi = AConfiguration_getDensity(h->app->config);
    if (dpi < 1 || dpi == ACONFIGURATION_DENSITY_ANY || dpi == ACONFIGURATION_DENSITY_NONE) {
        return 1.0f;
    }
    return (float)dpi / 160.0f;
}

static int rect_empty(const ARect *r) {
    return !r || r->right <= r->left || r->bottom <= r->top;
}

static JNIEnv *jenv(llr_and *h, int *did) {
    JavaVM *vm;
    JNIEnv *env = NULL;
    *did = 0;
    if (!h || !h->app || !h->app->activity || !h->app->activity->vm) {
        return NULL;
    }
    vm = h->app->activity->vm;
    if ((*vm)->GetEnv(vm, (void **)&env, JNI_VERSION_1_6) == JNI_OK) {
        return env;
    }
    if ((*vm)->AttachCurrentThread(vm, &env, NULL) != JNI_OK) {
        return NULL;
    }
    *did = 1;
    return env;
}

static void jdrop(llr_and *h, int did) {
    JavaVM *vm;
    if (!did || !h || !h->app || !h->app->activity) {
        return;
    }
    vm = h->app->activity->vm;
    if (vm) {
        (*vm)->DetachCurrentThread(vm);
    }
}

static int jni_clear(JNIEnv *env) {
    if (!env || !(*env)->ExceptionCheck(env)) {
        return 1;
    }
    (*env)->ExceptionClear(env);
    return 0;
}

static int dimen_px(JNIEnv *env, jobject act, const char *name) {
    jclass ac;
    jmethodID gr;
    jmethodID gi;
    jmethodID gd;
    jobject res;
    jclass rc;
    jstring jn;
    jstring jd;
    jstring ja;
    int id;
    ac = (*env)->GetObjectClass(env, act);
    gr = (*env)->GetMethodID(env, ac, "getResources", "()Landroid/content/res/Resources;");
    if (!gr || !jni_clear(env)) {
        return 0;
    }
    res = (*env)->CallObjectMethod(env, act, gr);
    if (!res || !jni_clear(env)) {
        return 0;
    }
    rc = (*env)->GetObjectClass(env, res);
    gi = (*env)->GetMethodID(env, rc, "getIdentifier", "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I");
    jn = (*env)->NewStringUTF(env, name);
    jd = (*env)->NewStringUTF(env, "dimen");
    ja = (*env)->NewStringUTF(env, "android");
    id = (int)(*env)->CallIntMethod(env, res, gi, jn, jd, ja);
    if (id == 0 || !jni_clear(env)) {
        return 0;
    }
    gd = (*env)->GetMethodID(env, rc, "getDimensionPixelSize", "(I)I");
    id = (int)(*env)->CallIntMethod(env, res, gd, id);
    if (!jni_clear(env)) {
        return 0;
    }
    return id < 0 ? 0 : id;
}

static int bar_px(llr_and *h, const char *name) {
    int did = 0;
    int n;
    JNIEnv *env = jenv(h, &did);
    if (!env || !h->app->activity->clazz) {
        jdrop(h, did);
        return 0;
    }
    n = dimen_px(env, h->app->activity->clazz, name);
    jdrop(h, did);
    return n;
}

static void note_size(llr_and *h) {
    ANativeWindow *win = h->app ? h->app->window : NULL;
    int ww;
    int wh;
    int cw;
    int ch;
    float sc;
    if (!win) {
        return;
    }
    ww = ANativeWindow_getWidth(win);
    wh = ANativeWindow_getHeight(win);
    if (ww < 1) {
        ww = 1;
    }
    if (wh < 1) {
        wh = 1;
    }
    cw = ww - h->il - h->ir;
    ch = wh - h->it - h->ib;
    if (cw < 1) {
        cw = 1;
    }
    if (ch < 1) {
        ch = 1;
    }
    sc = h->scale < 0.01f ? 1.0f : h->scale;
    h->log_w = (int)((float)cw / sc + 0.5f);
    h->log_h = (int)((float)ch / sc + 0.5f);
    if (h->log_w < 1) {
        h->log_w = 1;
    }
    if (h->log_h < 1) {
        h->log_h = 1;
    }
}

static void push_inset(llr_and *h) {
    llr_ui_ev ev;
    float sc = h->scale < 0.01f ? 1.0f : h->scale;
    memset(&ev, 0, sizeof ev);
    ev.kind = LLR_UI_INSET;
    ev.x = (float)h->il / sc;
    ev.y = (float)h->it / sc;
    ev.z = (float)h->ir / sc;
    ev.w = (float)h->ib / sc;
    llr_ui_push(h->q, &h->qn, ev);
}

static void insets_rect(llr_and *h, int ww, int wh) {
    ARect r = h->app->contentRect;
    h->il = r.left > 0 ? r.left : 0;
    h->it = r.top > 0 ? r.top : 0;
    h->ir = ww - r.right;
    h->ib = wh - r.bottom;
    if (h->ir < 0) {
        h->ir = 0;
    }
    if (h->ib < 0) {
        h->ib = 0;
    }
}

void llr_and_insets(llr_and *h) {
    ANativeWindow *win = h && h->app ? h->app->window : NULL;
    int ww;
    int wh;
    if (!win) {
        return;
    }
    ww = ANativeWindow_getWidth(win);
    wh = ANativeWindow_getHeight(win);
    if (rect_empty(&h->app->contentRect)) {
        h->il = 0;
        h->it = bar_px(h, "status_bar_height");
        h->ir = 0;
        h->ib = bar_px(h, "navigation_bar_height");
    } else {
        insets_rect(h, ww, wh);
    }
    note_size(h);
    push_inset(h);
}

static void logical_xy(llr_and *h, float x, float y, float *ox, float *oy) {
    float sc = h->scale < 0.01f ? 1.0f : h->scale;
    *ox = (x - (float)h->il) / sc;
    *oy = (y - (float)h->it) / sc;
}

static int mods_of(int m) {
    int o = 0;
    if (m & AMETA_SHIFT_ON) {
        o |= 1;
    }
    if (m & AMETA_CTRL_ON) {
        o |= 2;
    }
    if (m & AMETA_ALT_ON) {
        o |= 4;
    }
    if (m & AMETA_META_ON) {
        o |= 8;
    }
    return o;
}

static int motion_kind(int masked) {
    if (masked == AMOTION_EVENT_ACTION_DOWN || masked == AMOTION_EVENT_ACTION_POINTER_DOWN) {
        return LLR_UI_DOWN;
    }
    if (masked == AMOTION_EVENT_ACTION_MOVE) {
        return LLR_UI_MOVE;
    }
    if (masked == AMOTION_EVENT_ACTION_UP || masked == AMOTION_EVENT_ACTION_POINTER_UP) {
        return LLR_UI_UP;
    }
    if (masked == AMOTION_EVENT_ACTION_CANCEL) {
        return LLR_UI_CANCEL;
    }
    return -1;
}

static void on_motion(llr_and *h, const AInputEvent *ev) {
    int action = AMotionEvent_getAction(ev);
    int masked = action & AMOTION_EVENT_ACTION_MASK;
    int kind = motion_kind(masked);
    float x;
    float y;
    if (kind < 0) {
        return;
    }
    logical_xy(h, AMotionEvent_getX(ev, 0), AMotionEvent_getY(ev, 0), &x, &y);
    llr_ui_ptr(h->q, &h->qn, kind, x, y, 0.0f, 0.0f);
}

static void on_key(llr_and *h, const AInputEvent *ev) {
    int down = AKeyEvent_getAction(ev) == AKEY_EVENT_ACTION_DOWN;
    int uni = AKeyEvent_getUnicodeChar(ev);
    int mods = mods_of(AKeyEvent_getMetaState(ev));
    uint8_t buf[4];
    int n;
    llr_ui_key(h->q, &h->qn, down ? 1 : 0, uni, mods);
    if (!down || uni <= 0) {
        return;
    }
    n = llr_ui_utf8((uint32_t)uni, buf);
    llr_ui_text(h->q, &h->qn, buf, n);
}

int32_t llr_and_input(struct android_app *app, AInputEvent *ev) {
    llr_and *h = app ? app->userData : NULL;
    int type;
    if (!h || !ev) {
        return 0;
    }
    type = AInputEvent_getType(ev);
    if (type == AINPUT_EVENT_TYPE_MOTION) {
        on_motion(h, ev);
        return 1;
    }
    if (type == AINPUT_EVENT_TYPE_KEY) {
        on_key(h, ev);
        return 1;
    }
    return 0;
}

void llr_and_cmd(struct android_app *app, int32_t cmd) {
    llr_and *h = app ? app->userData : NULL;
    if (!h) {
        return;
    }
    if (cmd == APP_CMD_INIT_WINDOW || cmd == APP_CMD_WINDOW_RESIZED || cmd == APP_CMD_CONTENT_RECT_CHANGED) {
        h->scale = llr_and_density(h);
        llr_and_insets(h);
        llr_ui_kind(h->q, &h->qn, LLR_UI_FRAME);
        return;
    }
    if (cmd == APP_CMD_CONFIG_CHANGED || cmd == APP_CMD_RESUME) {
        h->scale = llr_and_density(h);
        llr_and_insets(h);
        if (cmd == APP_CMD_RESUME) {
            llr_and_life(h, LLR_UI_RESUME);
        }
        return;
    }
    if (cmd == APP_CMD_PAUSE) {
        llr_and_life(h, LLR_UI_PAUSE);
        return;
    }
    if (cmd == APP_CMD_DESTROY) {
        llr_and_life(h, LLR_UI_DESTROY);
    }
}

static jobject service(JNIEnv *env, jobject act, const char *name) {
    jclass cls = (*env)->GetObjectClass(env, act);
    jmethodID mid = (*env)->GetMethodID(env, cls, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;");
    jstring js;
    if (!mid) {
        return NULL;
    }
    js = (*env)->NewStringUTF(env, name);
    return (*env)->CallObjectMethod(env, act, mid, js);
}

static jobject clip_item(JNIEnv *env, jobject act) {
    jobject mgr = service(env, act, "clipboard");
    jclass mc;
    jmethodID mid;
    jobject seq;
    if (!mgr) {
        return NULL;
    }
    mc = (*env)->GetObjectClass(env, mgr);
    mid = (*env)->GetMethodID(env, mc, "getPrimaryClip", "()Landroid/content/ClipData;");
    seq = mid ? (*env)->CallObjectMethod(env, mgr, mid) : NULL;
    if (!seq || !jni_clear(env)) {
        return NULL;
    }
    mc = (*env)->GetObjectClass(env, seq);
    mid = (*env)->GetMethodID(env, mc, "getItemAt", "(I)Landroid/content/ClipData$Item;");
    return mid ? (*env)->CallObjectMethod(env, seq, mid, 0) : NULL;
}

static jstring clip_string(JNIEnv *env, jobject act) {
    jobject item = clip_item(env, act);
    jclass mc;
    jmethodID mid;
    jobject seq;
    if (!item) {
        return NULL;
    }
    mc = (*env)->GetObjectClass(env, item);
    mid = (*env)->GetMethodID(env, mc, "getText", "()Ljava/lang/CharSequence;");
    seq = mid ? (*env)->CallObjectMethod(env, item, mid) : NULL;
    if (!seq) {
        return NULL;
    }
    mc = (*env)->GetObjectClass(env, seq);
    mid = (*env)->GetMethodID(env, mc, "toString", "()Ljava/lang/String;");
    return mid ? (*env)->CallObjectMethod(env, seq, mid) : NULL;
}

static char *clip_copy(JNIEnv *env, jobject act, int *n) {
    jstring js = clip_string(env, act);
    const char *utf;
    char *buf;
    *n = 0;
    if (!js) {
        return NULL;
    }
    utf = (*env)->GetStringUTFChars(env, js, NULL);
    if (!utf) {
        return NULL;
    }
    *n = (int)strlen(utf);
    buf = malloc((size_t)*n + 1);
    if (buf) {
        memcpy(buf, utf, (size_t)*n + 1);
    }
    (*env)->ReleaseStringUTFChars(env, js, utf);
    return buf;
}

int64_t llr_ui_clip_get(void *p, uint8_t *buf, int64_t cap) {
    llr_and *h = p;
    int did = 0;
    int n = 0;
    char *text;
    JNIEnv *env;
    if (!h) {
        return 0;
    }
    env = jenv(h, &did);
    if (!env || !h->app->activity->clazz) {
        jdrop(h, did);
        return 0;
    }
    text = clip_copy(env, h->app->activity->clazz, &n);
    jdrop(h, did);
    if (!text) {
        return 0;
    }
    if (!buf || cap <= 0) {
        free(text);
        return n;
    }
    if ((int64_t)n > cap) {
        n = (int)cap;
    }
    memcpy(buf, text, (size_t)n);
    free(text);
    return n;
}

static jstring jbytes(JNIEnv *env, const uint8_t *bytes, int64_t n) {
    char *tmp;
    jstring js;
    if (n < 0) {
        n = 0;
    }
    tmp = malloc((size_t)n + 1);
    if (!tmp) {
        return NULL;
    }
    if (n > 0 && bytes) {
        memcpy(tmp, bytes, (size_t)n);
    }
    tmp[n] = 0;
    js = (*env)->NewStringUTF(env, tmp);
    free(tmp);
    return js;
}

void llr_ui_clip_set(void *p, const uint8_t *bytes, int64_t n) {
    llr_and *h = p;
    int did = 0;
    JNIEnv *env;
    jobject mgr;
    jclass cd;
    jmethodID mid;
    jstring label;
    jstring body;
    jobject data;
    if (!h) {
        return;
    }
    env = jenv(h, &did);
    if (!env || !h->app->activity->clazz) {
        jdrop(h, did);
        return;
    }
    mgr = service(env, h->app->activity->clazz, "clipboard");
    cd = (*env)->FindClass(env, "android/content/ClipData");
    mid = cd ? (*env)->GetStaticMethodID(env, cd, "newPlainText", "(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Landroid/content/ClipData;") : NULL;
    label = (*env)->NewStringUTF(env, "llr");
    body = jbytes(env, bytes, n);
    data = mid && body ? (*env)->CallStaticObjectMethod(env, cd, mid, label, body) : NULL;
    if (mgr && data) {
        jclass mc = (*env)->GetObjectClass(env, mgr);
        mid = (*env)->GetMethodID(env, mc, "setPrimaryClip", "(Landroid/content/ClipData;)V");
        if (mid) {
            (*env)->CallVoidMethod(env, mgr, mid, data);
        }
    }
    jni_clear(env);
    jdrop(h, did);
}

void llr_ui_text_input(void *p, int64_t on, float x, float y, float w, float hgt) {
    llr_and *h = p;
    (void)x;
    (void)y;
    (void)w;
    (void)hgt;
    if (!h || !h->app || !h->app->activity) {
        return;
    }
    if (on) {
        ANativeActivity_showSoftInput(h->app->activity, ANATIVEACTIVITY_SHOW_SOFT_INPUT_IMPLICIT);
        return;
    }
    ANativeActivity_hideSoftInput(h->app->activity, ANATIVEACTIVITY_HIDE_SOFT_INPUT_IMPLICIT_ONLY);
}
