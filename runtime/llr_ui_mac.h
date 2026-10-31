/* Cocoa host. The .m files are compiled with clang -fobjc-arc on Darwin only. */
#ifndef LLR_UI_MAC_H
#define LLR_UI_MAC_H

#import <Cocoa/Cocoa.h>

#import "llr_ui.h"

@class LlrHost;

@interface LlrView : NSView <NSTextInputClient>
@property (nonatomic, assign) LlrHost *owner;
@end

@interface LlrHost : NSObject <NSWindowDelegate>
@property (nonatomic, strong) NSWindow *win;
@property (nonatomic, strong) LlrView *view;
@property (nonatomic, assign) int logicalW;
@property (nonatomic, assign) int logicalH;
@property (nonatomic, assign) float scale;
@property (nonatomic, assign) int cursorId;
@property (nonatomic, assign) int textOn;
@property (nonatomic, assign) NSRect textRect;
@property (nonatomic, assign) uint8_t *px;
@property (nonatomic, assign) int pxW;
@property (nonatomic, assign) int pxH;
@property (nonatomic, assign) int pxStride;
- (instancetype)initWithW:(int64_t)w h:(int64_t)h;
- (void)shutdown;
- (void)pushKind:(int32_t)kind a:(int32_t)a b:(int32_t)b c:(int32_t)c x:(float)x y:(float)y z:(float)z w:(float)wv;
- (void)pushBytes:(const void *)bytes len:(int)len;
- (void)pushFrame;
- (void)noteSize;
- (void)noteScale;
- (void)requestFrame;
- (int64_t)pollInto:(uint8_t *)buf cap:(int64_t)cap;
- (NSCursor *)activeCursor;
@end

#endif
