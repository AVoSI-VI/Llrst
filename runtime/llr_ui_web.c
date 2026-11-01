/* No-op browser imports so a native link of a program that also
 * names the web host can resolve llr_web_*. The native arm never calls them. */
#include <stdint.h>

void llr_web_present(const uint8_t *px, int64_t w, int64_t h) {
    (void)px;
    (void)w;
    (void)h;
}

void llr_web_size(int32_t *w, int32_t *h) {
    if (w) {
        *w = 80;
    }
    if (h) {
        *h = 48;
    }
}

float llr_web_scale(void) {
    return 1.0f;
}

int64_t llr_web_now_ms(void) {
    return 0;
}

void llr_web_request_frame(void) {}

int32_t llr_web_poll_event(uint8_t *p) {
    (void)p;
    return 0;
}

void llr_web_set_cursor(int64_t id) {
    (void)id;
}

int32_t llr_web_clip_get(uint8_t *buf, int64_t cap) {
    (void)buf;
    (void)cap;
    return 0;
}

void llr_web_clip_set(const uint8_t *buf, int64_t n) {
    (void)buf;
    (void)n;
}

void llr_web_text_input(int64_t on, float x, float y, float w, float h) {
    (void)on;
    (void)x;
    (void)y;
    (void)w;
    (void)h;
}

void llr_web_quit(void) {}

void llr_web_bind(const uint8_t *p) {
    (void)p;
}

int32_t llr_web_app(void) {
    return 0;
}
