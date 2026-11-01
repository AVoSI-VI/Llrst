/* iOS window, display link, and present. Touches and text live in llr_ui_ios_in.m. */
#import "llr_ui_ios.h"

#import <QuartzCore/QuartzCore.h>
#import <stdlib.h>
#import <string.h>
#import <time.h>

static NSMutableArray<LlrIosHost *> *llr_live;
static LlrIosHost *g_host;

static void host_life(int kind) {
    if (g_host) {
        [g_host pushLife:kind];
    }
}

@implementation LlrIosView

- (void)layoutSubviews {
    [super layoutSubviews];
    if (self.owner) {
        [self.owner noteInsets];
    }
}

@end

@implementation LlrIosHost {
    llr_ui_ev queue[LLR_UI_RING];
    int qn;
    int firstFlag;
    int frameFlag;
    int dead;
}

- (void)pushKind:(int32_t)kind a:(int32_t)a x:(float)x y:(float)y z:(float)z w:(float)wv {
    llr_ui_ev ev;
    memset(&ev, 0, sizeof ev);
    ev.kind = kind;
    ev.a = a;
    ev.x = x;
    ev.y = y;
    ev.z = z;
    ev.w = wv;
    llr_ui_push(queue, &qn, ev);
}

- (void)pushBytes:(const void *)bytes len:(int)len {
    llr_ui_text(queue, &qn, bytes, len);
}

- (void)pushLife:(int)a {
    [self pushKind:LLR_UI_LIFE a:a x:0 y:0 z:0 w:0];
}

- (void)pushFrame {
    if (qn > 0 && queue[qn - 1].kind == LLR_UI_FRAME) {
        return;
    }
    llr_ui_kind(queue, &qn, LLR_UI_FRAME);
}

- (void)noteScale {
    float s = (float)UIScreen.mainScreen.scale;
    if (s < 0.01f) {
        s = 1;
    }
    self.scale = s;
    self.view.contentScaleFactor = s;
    self.pixels.contentsScale = s;
}

- (UIWindow *)makeWindow:(CGRect)frame {
    UIViewController *vc = [[UIViewController alloc] init];
    UIWindow *win = [[UIWindow alloc] initWithFrame:frame];
    self.view = [[LlrIosView alloc] initWithFrame:frame];
    self.view.owner = self;
    self.view.backgroundColor = UIColor.blackColor;
    self.view.multipleTouchEnabled = NO;
    self.pixels = [CALayer layer];
    self.pixels.contentsGravity = kCAGravityResize;
    [self.view.layer addSublayer:self.pixels];
    self.field = [[UITextField alloc] initWithFrame:CGRectZero];
    self.field.delegate = self;
    self.field.autocorrectionType = UITextAutocorrectionTypeNo;
    self.field.alpha = 0;
    [self.view addSubview:self.field];
    vc.view = self.view;
    win.rootViewController = vc;
    win.backgroundColor = UIColor.blackColor;
    return win;
}

- (instancetype)initWithW:(int64_t)w h:(int64_t)h {
    CGRect frame;
    (void)w;
    (void)h;
    self = [super init];
    if (!self) {
        return nil;
    }
    firstFlag = 1;
    self.scale = 1;
    self.logicalW = 1;
    self.logicalH = 1;
    frame = UIScreen.mainScreen.bounds;
    if (frame.size.width < 1 || frame.size.height < 1) {
        frame = CGRectMake(0, 0, 320, 480);
    }
    self.win = [self makeWindow:frame];
    [self noteScale];
    llr_ios_on_life(host_life);
    g_host = self;
    [self.win makeKeyAndVisible];
    [self noteInsets];
    return self;
}

- (void)shutdown {
    if (dead) {
        return;
    }
    dead = 1;
    [self.link invalidate];
    self.link = nil;
    self.view.owner = nil;
    self.field.delegate = nil;
    self.win.hidden = YES;
    if (g_host == self) {
        g_host = nil;
    }
}

- (void)tick:(CADisplayLink *)link {
    (void)link;
    [self.link invalidate];
    self.link = nil;
    frameFlag = 1;
}

- (void)requestFrame {
    if (self.link || dead) {
        return;
    }
    self.link = [CADisplayLink displayLinkWithTarget:self selector:@selector(tick:)];
    [self.link addToRunLoop:NSRunLoop.mainRunLoop forMode:NSRunLoopCommonModes];
}

- (int64_t)pollInto:(uint8_t *)buf cap:(int64_t)cap {
    if (dead) {
        return 0;
    }
    if (firstFlag) {
        firstFlag = 0;
        [self pushFrame];
        return llr_ui_copy(queue, &qn, buf, cap);
    }
    while (qn == 0 && !frameFlag && !dead) {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, 1.0, true);
    }
    if (frameFlag) {
        frameFlag = 0;
        [self pushFrame];
    }
    return llr_ui_copy(queue, &qn, buf, cap);
}

- (void)showPx:(const uint8_t *)src w:(int)w h:(int)hgt stride:(int)stride {
    CGDataProviderRef prov;
    CGColorSpaceRef cs;
    CGImageRef img;
    CFDataRef data;
    size_t n = (size_t)stride * (size_t)hgt;
    if (!src || w < 1 || hgt < 1) {
        return;
    }
    data = CFDataCreate(kCFAllocatorDefault, src, n);
    if (!data) {
        return;
    }
    prov = CGDataProviderCreateWithCFData(data);
    CFRelease(data);
    cs = CGColorSpaceCreateDeviceRGB();
    img = CGImageCreate((size_t)w, (size_t)hgt, 8, 32, (size_t)stride, cs,
        kCGBitmapByteOrder32Little | kCGImageAlphaPremultipliedFirst, prov, NULL, false, kCGRenderingIntentDefault);
    self.pixels.contents = (__bridge id)img;
    CGImageRelease(img);
    CGColorSpaceRelease(cs);
    CGDataProviderRelease(prov);
}

@end

static LlrIosHost *host_of(void *p) {
    return (__bridge LlrIosHost *)p;
}

void *llr_ui_open(int64_t w, int64_t h) {
    LlrIosHost *host = [[LlrIosHost alloc] initWithW:w h:h];
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
    LlrIosHost *h = host_of(p);
    if (!h) {
        return;
    }
    [h shutdown];
    [llr_live removeObject:h];
}

void llr_ui_quit(void *p) {
    LlrIosHost *h = host_of(p);
    if (h) {
        [h shutdown];
    }
    exit(0);
}

void llr_ui_size(void *p, int64_t *w, int64_t *h) {
    LlrIosHost *host = host_of(p);
    if (!host) {
        return;
    }
    if (w) {
        *w = host.logicalW > 0 ? host.logicalW : 1;
    }
    if (h) {
        *h = host.logicalH > 0 ? host.logicalH : 1;
    }
}

float llr_ui_scale(void *p) {
    LlrIosHost *h = host_of(p);
    if (!h || h.scale < 0.01f) {
        return 1.0f;
    }
    return h.scale;
}

void llr_ui_present(void *p, const uint8_t *px, int64_t w, int64_t hgt, int64_t stride) {
    LlrIosHost *h = host_of(p);
    if (!h || !px || w <= 0 || hgt <= 0 || stride <= 0) {
        return;
    }
    [h placePixels];
    [h showPx:px w:(int)w h:(int)hgt stride:(int)stride];
}

int64_t llr_ui_poll(void *p, uint8_t *buf, int64_t cap) {
    LlrIosHost *h = host_of(p);
    if (!h) {
        return 0;
    }
    return [h pollInto:buf cap:cap];
}

void llr_ui_request_frame(void *p) {
    LlrIosHost *h = host_of(p);
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

void llr_ui_set_cursor(void *p, int64_t id) {
    (void)p;
    (void)id;
}
