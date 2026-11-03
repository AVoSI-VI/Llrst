/* Android accessibility. The provider class is optional; a missing class only stores nothing. */
#include "llr_ui_android.h"
#include "llr_ui_a11y.h"

#include <jni.h>
#include <stdlib.h>
#include <string.h>

static llr_and *g_host;

JNIEXPORT void JNICALL Java_llr_ui_A11y_invoke(JNIEnv *env, jclass cls, jint id, jint action, jstring text) {
    const char *s = NULL;
    (void)cls;
    if (!g_host) {
        return;
    }
    if (text) {
        s = (*env)->GetStringUTFChars(env, text, NULL);
    }
    llr_ui_action(g_host->q, &g_host->qn, action, id, s, s ? (int)strlen(s) : 0);
    if (s) {
        (*env)->ReleaseStringUTFChars(env, text, s);
    }
}

static JNIEnv *env_of(llr_and *h) {
    JavaVM *vm;
    JNIEnv *env = NULL;
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
    return env;
}

static int failed(JNIEnv *env) {
    if (!(*env)->ExceptionCheck(env)) {
        return 0;
    }
    (*env)->ExceptionClear(env);
    return 1;
}

static jintArray ints_of(JNIEnv *env, const llr_a11y_tree *t, int field) {
    jintArray arr = (*env)->NewIntArray(env, (jsize)t->count);
    jint *buf;
    uint32_t i;
    if (!arr) {
        return NULL;
    }
    buf = (jint *)calloc(t->count ? t->count : 1, sizeof(jint));
    if (!buf) {
        return NULL;
    }
    for (i = 0; i < t->count; i++) {
        uint32_t v = 0;
        if (field == 0) {
            v = t->nodes[i].id;
        } else if (field == 1) {
            v = t->nodes[i].parent == 0xFFFFFFFFu ? (uint32_t)-1 : t->nodes[i].parent;
        } else if (field == 2) {
            v = t->nodes[i].role;
        } else if (field == 3) {
            v = t->nodes[i].flags;
        } else {
            v = t->nodes[i].actions;
        }
        buf[i] = (jint)v;
    }
    (*env)->SetIntArrayRegion(env, arr, 0, (jsize)t->count, buf);
    free(buf);
    return arr;
}

static jfloatArray rects_of(JNIEnv *env, const llr_a11y_tree *t) {
    jfloatArray arr = (*env)->NewFloatArray(env, (jsize)t->count * 4);
    jfloat *buf;
    uint32_t i;
    if (!arr) {
        return NULL;
    }
    buf = (jfloat *)calloc(t->count ? (size_t)t->count * 4 : 1, sizeof(jfloat));
    if (!buf) {
        return NULL;
    }
    for (i = 0; i < t->count; i++) {
        buf[i * 4] = t->nodes[i].x;
        buf[i * 4 + 1] = t->nodes[i].y;
        buf[i * 4 + 2] = t->nodes[i].w;
        buf[i * 4 + 3] = t->nodes[i].h;
    }
    (*env)->SetFloatArrayRegion(env, arr, 0, (jsize)t->count * 4, buf);
    free(buf);
    return arr;
}

static jobjectArray strs_of(JNIEnv *env, const llr_a11y_tree *t, int value) {
    jclass strc = (*env)->FindClass(env, "java/lang/String");
    jobjectArray arr;
    uint32_t i;
    if (!strc) {
        return NULL;
    }
    arr = (*env)->NewObjectArray(env, (jsize)t->count, strc, NULL);
    if (!arr) {
        return NULL;
    }
    for (i = 0; i < t->count; i++) {
        const char *s = value ? t->nodes[i].value : t->nodes[i].label;
        jstring js = (*env)->NewStringUTF(env, s ? s : "");
        (*env)->SetObjectArrayElement(env, arr, (jsize)i, js);
        (*env)->DeleteLocalRef(env, js);
    }
    return arr;
}

static jobject decor_of(JNIEnv *env, jobject activity) {
    jclass act;
    jclass win;
    jmethodID get_window;
    jmethodID get_decor;
    jobject window;
    jobject decor;
    act = (*env)->GetObjectClass(env, activity);
    get_window = (*env)->GetMethodID(env, act, "getWindow", "()Landroid/view/Window;");
    if (failed(env) || !get_window) {
        return NULL;
    }
    window = (*env)->CallObjectMethod(env, activity, get_window);
    if (failed(env) || !window) {
        return NULL;
    }
    win = (*env)->GetObjectClass(env, window);
    get_decor = (*env)->GetMethodID(env, win, "getDecorView", "()Landroid/view/View;");
    if (failed(env) || !get_decor) {
        return NULL;
    }
    decor = (*env)->CallObjectMethod(env, window, get_decor);
    if (failed(env)) {
        return NULL;
    }
    return decor;
}

static void publish(JNIEnv *env, llr_and *h, const llr_a11y_tree *t) {
    jclass cls;
    jmethodID set_tree;
    jmethodID attach;
    jobject decor;
    jintArray ids, parents, roles, flags, actions;
    jfloatArray rects;
    jobjectArray labels, values;
    cls = (*env)->FindClass(env, "llr/ui/A11y");
    if (!cls || failed(env)) {
        return;
    }
    set_tree = (*env)->GetStaticMethodID(env, cls, "setTree",
        "([I[I[I[I[I[F[Ljava/lang/String;[Ljava/lang/String;)V");
    attach = (*env)->GetStaticMethodID(env, cls, "attach", "(Landroid/view/View;)V");
    if (failed(env) || !set_tree || !attach) {
        return;
    }
    ids = ints_of(env, t, 0);
    parents = ints_of(env, t, 1);
    roles = ints_of(env, t, 2);
    flags = ints_of(env, t, 3);
    actions = ints_of(env, t, 4);
    rects = rects_of(env, t);
    labels = strs_of(env, t, 0);
    values = strs_of(env, t, 1);
    if (!ids || !parents || !roles || !flags || !actions || !rects || !labels || !values) {
        return;
    }
    (*env)->CallStaticVoidMethod(env, cls, set_tree, ids, parents, roles, flags, actions, rects, labels, values);
    if (failed(env)) {
        return;
    }
    decor = decor_of(env, h->app->activity->clazz);
    if (!decor) {
        return;
    }
    (*env)->CallStaticVoidMethod(env, cls, attach, decor);
    failed(env);
}

void llr_ui_a11y_update(void *raw, const uint8_t *bytes, int64_t n) {
    llr_and *h = (llr_and *)raw;
    llr_a11y_tree tree;
    JNIEnv *env;
    if (!h) {
        return;
    }
    g_host = h;
    memset(&tree, 0, sizeof tree);
    if (!llr_a11y_parse(&tree, bytes, n)) {
        return;
    }
    env = env_of(h);
    if (env) {
        publish(env, h, &tree);
    }
    llr_a11y_clear(&tree);
}
