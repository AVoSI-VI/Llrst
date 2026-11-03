/* iOS accessibility elements built from the semantics blob. */
#import "llr_ui_ios.h"
#import "llr_ui_a11y.h"

#import <objc/runtime.h>

static const void *llr_acc_key = &llr_acc_key;

@interface LlrIosEl : UIAccessibilityElement
@property (nonatomic, assign) int nodeId;
@property (nonatomic, assign) int actions;
@property (nonatomic, assign) LlrIosHost *host;
@end

@implementation LlrIosEl

- (BOOL)accessibilityActivate {
    int bit = (self.actions & 8) ? 8 : 1;
    if (self.host) {
        [self.host pushAction:bit node:self.nodeId text:NULL len:0];
    }
    return YES;
}

- (void)accessibilityElementDidBecomeFocused {
    if (self.host) {
        [self.host pushAction:2 node:self.nodeId text:NULL len:0];
    }
}

@end

static uint64_t traits_of(uint32_t role, uint32_t flags) {
    uint64_t t = UIAccessibilityTraitNone;
    if (role == 1 || role == 8) {
        t |= UIAccessibilityTraitButton;
    }
    if (role == 2 || role == 3) {
        t |= UIAccessibilityTraitButton;
    }
    if (flags & 2) {
        t |= UIAccessibilityTraitSelected;
    }
    if (role == 13 || role == 6 || role == 12) {
        t |= UIAccessibilityTraitStaticText;
    }
    if (role == 14) {
        t |= UIAccessibilityTraitImage;
    }
    if (flags & 1) {
        t |= UIAccessibilityTraitAllowsDirectInteraction;
    }
    return t;
}

static LlrIosEl *make_el(LlrIosHost *host, const llr_a11y_node *n) {
    LlrIosEl *el = [[LlrIosEl alloc] initWithAccessibilityContainer:host.view];
    CGRect frame;
    NSString *label = n->label && n->label[0] ? [NSString stringWithUTF8String:n->label] : @"";
    el.nodeId = (int)n->id;
    el.actions = (int)n->actions;
    el.host = host;
    el.accessibilityLabel = label;
    if (n->value && n->value[0]) {
        el.accessibilityValue = [NSString stringWithUTF8String:n->value];
    }
    el.accessibilityTraits = traits_of(n->role, n->flags);
    frame = CGRectMake(n->x, n->y, n->w, n->h);
    el.accessibilityFrameInContainerSpace = frame;
    return el;
}

void llr_ui_a11y_update(void *raw, const uint8_t *bytes, int64_t n) {
    LlrIosHost *host = (__bridge LlrIosHost *)raw;
    llr_a11y_tree tree;
    NSMutableArray *els;
    uint32_t i;
    if (!host || !host.view) {
        return;
    }
    memset(&tree, 0, sizeof tree);
    if (!llr_a11y_parse(&tree, bytes, n)) {
        objc_setAssociatedObject(host.view, llr_acc_key, nil, OBJC_ASSOCIATION_RETAIN);
        return;
    }
    els = [NSMutableArray arrayWithCapacity:tree.count];
    for (i = 0; i < tree.count; i++) {
        [els addObject:make_el(host, &tree.nodes[i])];
    }
    llr_a11y_clear(&tree);
    objc_setAssociatedObject(host.view, llr_acc_key, els, OBJC_ASSOCIATION_RETAIN);
    UIAccessibilityPostNotification(UIAccessibilityLayoutChangedNotification, nil);
}

@implementation LlrIosView (LlrA11y)

- (BOOL)isAccessibilityElement {
    return NO;
}

- (NSArray *)accessibilityElements {
    NSArray *els = objc_getAssociatedObject(self, llr_acc_key);
    return els ? els : @[];
}

@end
