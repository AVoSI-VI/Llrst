/* iOS host. The .m files are compiled with clang -fobjc-arc for the iOS SDK. */
#ifndef LLR_UI_IOS_H
#define LLR_UI_IOS_H

#import "llr_ui.h"

#import <UIKit/UIKit.h>

@class LlrIosHost;

@interface LlrIosView : UIView
@property (nonatomic, assign) LlrIosHost *owner;
@end

@interface LlrIosHost : NSObject <UITextFieldDelegate>
@property (nonatomic, strong) UIWindow *win;
@property (nonatomic, strong) LlrIosView *view;
@property (nonatomic, strong) CALayer *pixels;
@property (nonatomic, strong) UITextField *field;
@property (nonatomic, strong) CADisplayLink *link;
@property (nonatomic, assign) int logicalW;
@property (nonatomic, assign) int logicalH;
@property (nonatomic, assign) float scale;
@property (nonatomic, assign) float insetL;
@property (nonatomic, assign) float insetT;
@property (nonatomic, assign) float insetR;
@property (nonatomic, assign) float insetB;
@property (nonatomic, assign) int insetSet;
- (instancetype)initWithW:(int64_t)w h:(int64_t)h;
- (void)shutdown;
- (void)pushKind:(int32_t)kind a:(int32_t)a x:(float)x y:(float)y z:(float)z w:(float)wv;
- (void)pushBytes:(const void *)bytes len:(int)len;
- (void)pushLife:(int)a;
- (void)pushFrame;
- (void)noteScale;
- (void)noteInsets;
- (void)placePixels;
- (void)showPx:(const uint8_t *)src w:(int)w h:(int)hgt stride:(int)stride;
- (void)requestFrame;
- (int64_t)pollInto:(uint8_t *)buf cap:(int64_t)cap;
@end

void llr_ios_on_life(void (*fn)(int));

#endif
