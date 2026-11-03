/* Cocoa window. poll pumps NSApp; it does not call [NSApp run]. */
#import "llr_ui_mac.h"

#import <time.h>
#import <stdlib.h>

static NSMutableArray<LlrHost *> *llr_live;

static void ensure_app(void) {
    static int done = 0;
    if (done) {
        return;
    }
    done = 1;
    [NSApplication sharedApplication];
    [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
}

@implementation LlrHost {
    llr_ui_ev queue[LLR_UI_RING];
    int qn;
    int firstFlag;
    int frameFlag;
    int dead;
}

- (instancetype)initWithW:(int64_t)w h:(int64_t)h {
    NSRect frame;
    self = [super init];
    if (!self) {
        return nil;
    }
    if (w < 1) {
        w = 1;
    }
    if (h < 1) {
        h = 1;
    }
    _logicalW = (int)w;
    _logicalH = (int)h;
    _scale = 1;
    firstFlag = 1;
    ensure_app();
    frame = NSMakeRect(0, 0, (CGFloat)w, (CGFloat)h);
    _win = [[NSWindow alloc] initWithContentRect:frame
                                       styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskMiniaturizable | NSWindowStyleMaskResizable
                                         backing:NSBackingStoreBuffered
                                           defer:NO];
    _win.releasedWhenClosed = NO;
    _win.acceptsMouseMovedEvents = YES;
    _view = [[LlrView alloc] initWithFrame:frame];
    _view.owner = self;
    _view.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    _win.contentView = _view;
    _win.delegate = self;
    _win.title = @"llr";
    _scale = (float)_win.backingScaleFactor;
    if (_scale < 0.01f) {
        _scale = 1;
    }
    [_win makeFirstResponder:_view];
    [_win makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
    return self;
}

- (void)shutdown {
    if (dead) {
        return;
    }
    dead = 1;
    self.view.owner = nil;
    [self.win orderOut:nil];
    self.win.delegate = nil;
    self.win = nil;
    self.view = nil;
    free(self.px);
    self.px = NULL;
}

- (void)pushKind:(int32_t)kind a:(int32_t)a b:(int32_t)b c:(int32_t)c x:(float)x y:(float)y z:(float)z w:(float)wv {
    llr_ui_ev ev;
    memset(&ev, 0, sizeof ev);
    ev.kind = kind;
    ev.a = a;
    ev.b = b;
    ev.c = c;
    ev.x = x;
    ev.y = y;
    ev.z = z;
    ev.w = wv;
    llr_ui_push(queue, &qn, ev);
}

- (void)pushAction:(int32_t)action node:(int32_t)node text:(const void *)bytes len:(int)len {
    llr_ui_action(queue, &qn, action, node, bytes, len);
}

- (void)pushBytes:(const void *)bytes len:(int)len {
    llr_ui_text(queue, &qn, bytes, len);
}

- (void)pushFrame {
    if (qn > 0 && queue[qn - 1].kind == LLR_UI_FRAME) {
        return;
    }
    llr_ui_kind(queue, &qn, LLR_UI_FRAME);
}

- (void)noteSize {
    NSRect r = self.view.bounds;
    int w = (int)(r.size.width + 0.5);
    int h = (int)(r.size.height + 0.5);
    if (w < 1) {
        w = 1;
    }
    if (h < 1) {
        h = 1;
    }
    self.logicalW = w;
    self.logicalH = h;
    [self pushFrame];
}

- (void)noteScale {
    self.scale = (float)self.win.backingScaleFactor;
    if (self.scale < 0.01f) {
        self.scale = 1;
    }
    [self pushFrame];
}

- (void)requestFrame {
    frameFlag = 1;
}

- (void)drainUntil:(NSDate *)until {
    for (;;) {
        NSEvent *ev = [NSApp nextEventMatchingMask:NSEventMaskAny untilDate:until inMode:NSDefaultRunLoopMode dequeue:YES];
        if (!ev) {
            return;
        }
        [NSApp sendEvent:ev];
        until = [NSDate distantPast];
    }
}

- (int64_t)pollInto:(uint8_t *)buf cap:(int64_t)cap {
    [self drainUntil:[NSDate distantPast]];
    if (firstFlag) {
        firstFlag = 0;
        [self pushFrame];
        return llr_ui_copy(queue, &qn, buf, cap);
    }
    if (qn == 0 && !frameFlag) {
        [self drainUntil:[NSDate distantFuture]];
    }
    if (frameFlag) {
        frameFlag = 0;
        [self pushFrame];
    }
    return llr_ui_copy(queue, &qn, buf, cap);
}

- (NSCursor *)activeCursor {
    switch (self.cursorId) {
    case LLR_UI_CUR_TEXT:
        return [NSCursor IBeamCursor];
    case LLR_UI_CUR_HAND:
        return [NSCursor pointingHandCursor];
    case LLR_UI_CUR_CROSS:
        return [NSCursor crosshairCursor];
    default:
        return [NSCursor arrowCursor];
    }
}

- (BOOL)windowShouldClose:(NSWindow *)sender {
    (void)sender;
    [self pushKind:LLR_UI_QUIT a:0 b:0 c:0 x:0 y:0 z:0 w:0];
    return NO;
}

- (void)windowDidResize:(NSNotification *)note {
    (void)note;
    [self noteSize];
}

- (void)windowDidChangeBackingProperties:(NSNotification *)note {
    (void)note;
    [self noteScale];
}

@end

static LlrHost *host_of(void *p) {
    return (__bridge LlrHost *)p;
}

void *llr_ui_open(int64_t w, int64_t h) {
    LlrHost *host = [[LlrHost alloc] initWithW:w h:h];
    if (!host) {
        return NULL;
    }
    if (!llr_live) {
        llr_live = [NSMutableArray new];
    }
    [llr_live addObject:host];
    return (__bridge void *)host;
}

void llr_ui_close(void *p) {
    LlrHost *h = host_of(p);
    if (!h) {
        return;
    }
    [h shutdown];
    [llr_live removeObject:h];
}

void llr_ui_quit(void *p) {
    llr_ui_close(p);
}

void llr_ui_size(void *p, int64_t *w, int64_t *h) {
    LlrHost *host = host_of(p);
    if (!host) {
        return;
    }
    if (w) {
        *w = host.logicalW;
    }
    if (h) {
        *h = host.logicalH;
    }
}

float llr_ui_scale(void *p) {
    LlrHost *h = host_of(p);
    if (!h || h.scale < 0.01f) {
        return 1.0f;
    }
    return h.scale;
}

void llr_ui_present(void *p, const uint8_t *px, int64_t w, int64_t hgt, int64_t stride) {
    LlrHost *h = host_of(p);
    size_t n;
    if (!h || !px || w <= 0 || hgt <= 0 || stride <= 0) {
        return;
    }
    n = (size_t)stride * (size_t)hgt;
    if (h.pxW != (int)w || h.pxH != (int)hgt || h.pxStride != (int)stride) {
        free(h.px);
        h.px = malloc(n);
        h.pxW = (int)w;
        h.pxH = (int)hgt;
        h.pxStride = (int)stride;
    }
    if (!h.px) {
        return;
    }
    memcpy(h.px, px, n);
    [h.view setNeedsDisplay:YES];
    [h.view displayIfNeeded];
}

int64_t llr_ui_poll(void *p, uint8_t *buf, int64_t cap) {
    LlrHost *h = host_of(p);
    if (!h) {
        return 0;
    }
    return [h pollInto:buf cap:cap];
}

void llr_ui_request_frame(void *p) {
    LlrHost *h = host_of(p);
    if (h) {
        [h requestFrame];
    }
}

int64_t llr_ui_now_ms(void *p) {
    struct timespec ts;
    (void)p;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (int64_t)ts.tv_sec * 1000 + (int64_t)ts.tv_nsec / 1000000;
}
