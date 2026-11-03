/* Process-local slots for a reloaded UI library. The exe and the library each have a copy. */
#include <stdint.h>

static int64_t g_prev;
static int64_t g_app;
static int64_t g_code;
static int64_t g_graft;
static int64_t g_bad;

int64_t llr_reload_prev(void) { return g_prev; }

void llr_reload_set_prev(int64_t v) { g_prev = v; }

void llr_reload_set_app(int64_t v) { g_app = v; }

int64_t llr_reload_app(void) { return g_app; }

void llr_reload_set_code(int64_t v) { g_code = v; }

int64_t llr_reload_code(void) { return g_code; }

int64_t llr_reload_graft(void) { return g_graft; }

void llr_reload_set_graft(int64_t v) { g_graft = v; }

int64_t llr_reload_bad(void) { return g_bad; }

void llr_reload_set_bad(int64_t v) { g_bad = v; }
