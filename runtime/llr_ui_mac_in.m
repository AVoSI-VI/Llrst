/* Cocoa pointer, key, committed text, pasteboard, and cursor. */
#import "llr_ui_mac.h"

#import <string.h>

static int mods_of(NSEvent *ev) {
    NSEventModifierFlags f = ev.modifierFlags;
    int m = 0;
    if (f & NSEventModifierFlagShift) {
        m |= 1;
    }
    if (f & NSEventModifierFlagControl) {
        m |= 2;
    }
    if (f & NSEventModifierFlagOption) {
        m |= 4;
    }
    if (f & NSEventModifierFlagCommand) {
        m |= 8;
    }
    return m;
}

static int code_of(NSEvent *ev) {
    NSString *s = ev.charactersIgnoringModifiers;
    unichar c;
    if (s.length == 0) {
        return (int)ev.keyCode;
    }
    c = [s characterAtIndex:0];
    if (c == 0x7F) {
        return 8;
    }
    if (c == 3 || c == 13) {
        return 13;
    }
    if (c < 0xF700) {
        return (int)c;
    }
    return (int)ev.keyCode;
}

static void push_ns(LlrHost *h, NSString *s) {
    const char *u;
    if (!h || s.length == 0) {
        return;
    }
    u = s.UTF8String;
    if (!u) {
        return;
    }
    [h pushBytes:u len:(int)strlen(u)];
}

@implementation LlrView

- (BOOL)isFlipped {
    return YES;
}

- (BOOL)acceptsFirstResponder {
    return YES;
}

- (void)drawRect:(NSRect)dirty {
    LlrHost *h = self.owner;
    unsigned char *planes[1];
    NSBitmapImageRep *rep;
    (void)dirty;
    if (!h || !h.px || h.pxW <= 0 || h.pxH <= 0) {
        return;
    }
    planes[0] = h.px;
    rep = [[NSBitmapImageRep alloc] initWithBitmapDataPlanes:planes
                                                   pixelsWide:h.pxW
                                                   pixelsHigh:h.pxH
                                                bitsPerSample:8
                                              samplesPerPixel:4
                                                     hasAlpha:YES
                                                     isPlanar:NO
                                               colorSpaceName:NSDeviceRGBColorSpace
                                                 bitmapFormat:NSBitmapFormatThirtyTwoBitLittleEndian | NSBitmapFormatAlphaFirst
                                                  bytesPerRow:h.pxStride
                                                 bitsPerPixel:32];
    if (rep) {
        [rep drawInRect:self.bounds];
    }
}

- (NSPoint)logicalPoint:(NSEvent *)ev {
    return [self convertPoint:ev.locationInWindow fromView:nil];
}

- (void)mouseDown:(NSEvent *)ev {
    NSPoint p = [self logicalPoint:ev];
    [self.owner pushKind:LLR_UI_POINTER a:LLR_UI_DOWN b:0 c:0 x:(float)p.x y:(float)p.y z:0 w:0];
}

- (void)mouseDragged:(NSEvent *)ev {
    NSPoint p = [self logicalPoint:ev];
    [self.owner pushKind:LLR_UI_POINTER a:LLR_UI_MOVE b:0 c:0 x:(float)p.x y:(float)p.y z:0 w:0];
}

- (void)mouseMoved:(NSEvent *)ev {
    NSPoint p = [self logicalPoint:ev];
    [self.owner pushKind:LLR_UI_POINTER a:LLR_UI_MOVE b:0 c:0 x:(float)p.x y:(float)p.y z:0 w:0];
}

- (void)mouseUp:(NSEvent *)ev {
    NSPoint p = [self logicalPoint:ev];
    [self.owner pushKind:LLR_UI_POINTER a:LLR_UI_UP b:0 c:0 x:(float)p.x y:(float)p.y z:0 w:0];
}

- (void)scrollWheel:(NSEvent *)ev {
    NSPoint p = [self logicalPoint:ev];
    [self.owner pushKind:LLR_UI_POINTER a:LLR_UI_SCROLL b:0 c:0 x:(float)p.x y:(float)p.y z:(float)ev.scrollingDeltaX w:(float)ev.scrollingDeltaY];
}

- (void)keyDown:(NSEvent *)ev {
    LlrHost *h = self.owner;
    int code = code_of(ev);
    unichar c;
    if (!h) {
        return;
    }
    if (code != 0) {
        [h pushKind:LLR_UI_KEY a:1 b:code c:mods_of(ev) x:0 y:0 z:0 w:0];
    }
    if (h.textOn) {
        [self interpretKeyEvents:@[ev]];
        return;
    }
    if (ev.characters.length == 0) {
        return;
    }
    c = [ev.characters characterAtIndex:0];
    if (c >= 32 && c < 0xF700) {
        push_ns(h, ev.characters);
    }
}

- (void)keyUp:(NSEvent *)ev {
    int code = code_of(ev);
    if (self.owner && code != 0) {
        [self.owner pushKind:LLR_UI_KEY a:0 b:code c:mods_of(ev) x:0 y:0 z:0 w:0];
    }
}

- (void)insertText:(id)string replacementRange:(NSRange)range {
    NSString *s = [string isKindOfClass:[NSAttributedString class]] ? [(NSAttributedString *)string string] : string;
    (void)range;
    push_ns(self.owner, s);
}

- (void)setMarkedText:(id)string selectedRange:(NSRange)selected replacementRange:(NSRange)replacement {
    (void)string;
    (void)selected;
    (void)replacement;
}

- (void)unmarkText {
}

- (NSRange)selectedRange {
    return NSMakeRange(0, 0);
}

- (NSRange)markedRange {
    return NSMakeRange(NSNotFound, 0);
}

- (BOOL)hasMarkedText {
    return NO;
}

- (NSAttributedString *)attributedSubstringForProposedRange:(NSRange)range actualRange:(NSRangePointer)actual {
    (void)range;
    if (actual) {
        *actual = NSMakeRange(NSNotFound, 0);
    }
    return nil;
}

- (NSArray<NSAttributedStringKey> *)validAttributesForMarkedText {
    return @[];
}

- (NSRect)firstRectForCharacterRange:(NSRange)range actualRange:(NSRangePointer)actual {
    NSRect r = self.owner.textRect;
    (void)range;
    if (actual) {
        *actual = range;
    }
    r = [self convertRect:r toView:nil];
    return [self.window convertRectToScreen:r];
}

- (NSUInteger)characterIndexForPoint:(NSPoint)point {
    (void)point;
    return 0;
}

- (void)resetCursorRects {
    NSCursor *c = [self.owner activeCursor];
    if (c) {
        [self addCursorRect:self.bounds cursor:c];
    }
}

@end

void llr_ui_set_cursor(void *p, int64_t id) {
    LlrHost *h = (__bridge LlrHost *)p;
    NSCursor *c;
    if (!h) {
        return;
    }
    h.cursorId = (int)id;
    c = [h activeCursor];
    [c set];
    if (h.view) {
        [h.win invalidateCursorRectsForView:h.view];
    }
}

void llr_ui_text_input(void *p, int64_t on, float x, float y, float w, float hgt) {
    LlrHost *h = (__bridge LlrHost *)p;
    if (!h) {
        return;
    }
    h.textOn = on != 0;
    h.textRect = NSMakeRect(x, y, w, hgt);
    if (on && h.view) {
        [h.win makeFirstResponder:h.view];
    }
}

int64_t llr_ui_clip_get(void *p, uint8_t *buf, int64_t cap) {
    NSString *s;
    const char *u;
    int n;
    (void)p;
    s = [[NSPasteboard generalPasteboard] stringForType:NSPasteboardTypeString];
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
    NSPasteboard *pb = [NSPasteboard generalPasteboard];
    NSString *s;
    (void)p;
    if (n < 0) {
        n = 0;
    }
    s = [[NSString alloc] initWithBytes:bytes length:(NSUInteger)n encoding:NSUTF8StringEncoding];
    if (!s) {
        s = @"";
    }
    [pb clearContents];
    [pb setString:s forType:NSPasteboardTypeString];
}
