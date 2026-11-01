/* iOS touches, safe area, pasteboard, and the hidden text field. */
#import "llr_ui_ios.h"

#import <string.h>

@implementation LlrIosHost (llr_text)

- (void)placePixels {
    UIEdgeInsets in = self.view.safeAreaInsets;
    CGFloat w = self.logicalW > 0 ? self.logicalW : 1;
    CGFloat h = self.logicalH > 0 ? self.logicalH : 1;
    self.pixels.frame = CGRectMake(in.left, in.top, w, h);
    self.pixels.contentsScale = self.scale > 0.01f ? self.scale : 1;
}

- (void)noteInsets {
    UIEdgeInsets in = self.view.safeAreaInsets;
    CGRect b = self.view.bounds;
    CGFloat cw = b.size.width - in.left - in.right;
    CGFloat ch = b.size.height - in.top - in.bottom;
    int same;
    if (cw < 1) {
        cw = 1;
    }
    if (ch < 1) {
        ch = 1;
    }
    self.logicalW = (int)(cw + 0.5);
    self.logicalH = (int)(ch + 0.5);
    same = self.insetSet && self.insetL == (float)in.left && self.insetT == (float)in.top && self.insetR == (float)in.right && self.insetB == (float)in.bottom;
    self.insetL = (float)in.left;
    self.insetT = (float)in.top;
    self.insetR = (float)in.right;
    self.insetB = (float)in.bottom;
    self.insetSet = 1;
    [self placePixels];
    if (!same) {
        [self pushKind:LLR_UI_INSET a:0 x:self.insetL y:self.insetT z:self.insetR w:self.insetB];
    }
}

- (BOOL)textField:(UITextField *)field shouldChangeCharactersInRange:(NSRange)range replacementString:(NSString *)s {
    const char *u;
    (void)field;
    (void)range;
    u = s.UTF8String;
    if (u && u[0]) {
        [self pushBytes:u len:(int)strlen(u)];
    }
    return YES;
}

- (BOOL)textFieldShouldReturn:(UITextField *)field {
    (void)field;
    [self pushBytes:"\n" len:1];
    return NO;
}

@end

@implementation LlrIosView (llr_touch)

- (void)emit:(NSSet<UITouch *> *)touches kind:(int)kind {
    UITouch *t = touches.anyObject;
    CGPoint p;
    UIEdgeInsets in;
    if (!t || !self.owner) {
        return;
    }
    p = [t locationInView:self];
    in = self.safeAreaInsets;
    [self.owner pushKind:LLR_UI_POINTER a:kind x:(float)(p.x - in.left) y:(float)(p.y - in.top) z:0 w:0];
}

- (void)touchesBegan:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    (void)event;
    [self emit:touches kind:LLR_UI_DOWN];
}

- (void)touchesMoved:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    (void)event;
    [self emit:touches kind:LLR_UI_MOVE];
}

- (void)touchesEnded:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    (void)event;
    [self emit:touches kind:LLR_UI_UP];
}

- (void)touchesCancelled:(NSSet<UITouch *> *)touches withEvent:(UIEvent *)event {
    (void)event;
    [self emit:touches kind:LLR_UI_CANCEL];
}

@end

int64_t llr_ui_clip_get(void *p, uint8_t *buf, int64_t cap) {
    NSString *s = UIPasteboard.generalPasteboard.string;
    const char *u;
    int n;
    (void)p;
    if (!s) {
        return 0;
    }
    u = s.UTF8String;
    if (!u) {
        return 0;
    }
    n = (int)strlen(u);
    if (!buf || cap <= 0) {
        return n;
    }
    if ((int64_t)n > cap) {
        n = (int)cap;
    }
    memcpy(buf, u, (size_t)n);
    return n;
}

void llr_ui_clip_set(void *p, const uint8_t *bytes, int64_t n) {
    NSString *s;
    (void)p;
    if (n < 0) {
        n = 0;
    }
    s = [[NSString alloc] initWithBytes:bytes length:(NSUInteger)n encoding:NSUTF8StringEncoding];
    UIPasteboard.generalPasteboard.string = s ? s : @"";
}

void llr_ui_text_input(void *p, int64_t on, float x, float y, float w, float hgt) {
    LlrIosHost *h = (__bridge LlrIosHost *)p;
    UIEdgeInsets in;
    if (!h || !h.field) {
        return;
    }
    in = h.view.safeAreaInsets;
    h.field.frame = CGRectMake(in.left + x, in.top + y, w > 1 ? w : 1, hgt > 1 ? hgt : 1);
    h.field.alpha = on ? 0.02 : 0;
    if (on) {
        [h.field becomeFirstResponder];
        return;
    }
    [h.field resignFirstResponder];
}
