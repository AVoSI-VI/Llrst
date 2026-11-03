/* Loads a UI library and calls the three entry points. The tool does not call through a raw pointer. */
#include <stdint.h>
#include <string.h>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

void *llr_dlopen(const char *path) {
    wchar_t wide[1024];
    if (path == 0) {
        return 0;
    }
    if (MultiByteToWideChar(CP_UTF8, 0, path, -1, wide, 1024) == 0) {
        return 0;
    }
    return (void *)LoadLibraryW(wide);
}

void *llr_dlsym(void *lib, const char *name) {
    if (lib == 0 || name == 0) {
        return 0;
    }
    return (void *)GetProcAddress((HMODULE)lib, name);
}

void llr_dlclose(void *lib) {
    if (lib != 0) {
        FreeLibrary((HMODULE)lib);
    }
}
#else
#include <dlfcn.h>

void *llr_dlopen(const char *path) {
    if (path == 0) {
        return 0;
    }
    return dlopen(path, RTLD_NOW | RTLD_LOCAL);
}

void *llr_dlsym(void *lib, const char *name) {
    if (lib == 0 || name == 0) {
        return 0;
    }
    return dlsym(lib, name);
}

void llr_dlclose(void *lib) {
    if (lib != 0) {
        dlclose(lib);
    }
}
#endif

typedef int64_t (*boot_fn)(int64_t, int64_t, void *);
typedef void (*frame_fn)(void);
typedef void (*drop_fn)(int64_t);
typedef int64_t (*app_fn)(void);

int64_t llr_dl_boot(void *lib, int64_t prev, int64_t argc, void *argv) {
    boot_fn fn;
    if (lib == 0) {
        return -1;
    }
    fn = (boot_fn)llr_dlsym(lib, "llr_ui_boot");
    if (fn == 0) {
        return -1;
    }
    return fn(prev, argc, argv);
}

void llr_dl_frame(void *lib) {
    frame_fn fn;
    if (lib == 0) {
        return;
    }
    fn = (frame_fn)llr_dlsym(lib, "llr_ui_frame");
    if (fn != 0) {
        fn();
    }
}

void llr_dl_drop(void *lib, int64_t app) {
    drop_fn fn;
    if (lib == 0) {
        return;
    }
    fn = (drop_fn)llr_dlsym(lib, "llr_ui_drop");
    if (fn != 0) {
        fn(app);
    }
}

int64_t llr_dl_app(void *lib) {
    app_fn fn;
    if (lib == 0) {
        return 0;
    }
    fn = (app_fn)llr_dlsym(lib, "llr_reload_app");
    if (fn == 0) {
        return 0;
    }
    return fn();
}
