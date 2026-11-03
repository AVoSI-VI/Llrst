/* Milliseconds for fs.now_ms. Monotonic on both hosts. */
#include <stdint.h>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

int64_t llr_now_ms(void) { return (int64_t)GetTickCount64(); }
#else
#include <time.h>

int64_t llr_now_ms(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
}
#endif
