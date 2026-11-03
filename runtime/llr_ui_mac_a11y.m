/* Cocoa accessibility elements built from the semantics blob. */
#import "llr_ui_mac.h"
#import "llr_ui_a11y.h"

#import <objc/runtime.h>

static const void *llr_acc_key = &llr_acc_key;

@interface LlrAccEl : NSAccessibilityElement
@property (nonatomic, assign) int nodeId;
@property (nonatomic, assign) int actions;
@property (nonatomic, assign) LlrHost *host;
@end

@implementation LlrAccEl

- (BOOL)accessibilityPerformPress {
    int bit = (self.actions & 8) ? 8 : 1;
    if (self.host) {
        [self.host pushAction:bit node:self.nodeId text:NULL len:0];
    }
    return YES;
}

- (void)setAccessibilityFocused:(BOOL)on {
    if (on && self.host) {
        [self.host pushAction:2 node:self.nodeId text:NULL len:0];
    }
}

- (void)setAccessibilityValue:(id)value {
    NSString *s = [value isKindOfClass:[NSString class]] ? value : @"";
    const char *u = s.UTF8String ? s.UTF8String : "";
    if (self.host) {
        [self.host pushAction:4 node:self.nodeId text:u len:(int)strlen(u)];
    }
}

@end

static NSString *role_of(uint32_t role) {
    switch (role) {
    case 1: return NSAccessibilityButtonRole;
    case 2: return NSAccessibilityCheckBoxRole;
    case 3: return NSAccessibilityCheckBoxRole;
    case 4: return NSAccessibilitySliderRole;
    case 5: return NSAccessibilityTextFieldRole;
    case 11: return NSAccessibilityProgressIndicatorRole;
    case 14: return NSAccessibilityImageRole;
    default: return NSAccessibilityStaticTextRole;
    }
}

static LlrAccEl *make_el(LlrHost *host, const llr_a11y_node *n) {
    LlrAccEl *el = [LlrAccEl new];
    NSString *label = n->label && n->label[0] ? [NSString stringWithUTF8String:n->label] : @"";
    NSRect local;
    NSRect screen;
    el.nodeId = (int)n->id;
    el.actions = (int)n->actions;
    el.host = host;
    el.accessibilityRole = role_of(n->role);
    el.accessibilityLabel = label;
    if (n->value && n->value[0]) {
        el.accessibilityValue = [NSString stringWithUTF8String:n->value];
    }
    local = NSMakeRect(n->x, host.view.bounds.size.height - n->y - n->h, n->w, n->h);
    screen = [host.win convertRectToScreen:[host.view convertRect:local toView:nil]];
    el.accessibilityFrame = screen;
    el.accessibilityParent = host.view;
    return el;
}

void llr_ui_a11y_update(void *raw, const uint8_t *bytes, int64_t n) {
    LlrHost *host = (__bridge LlrHost *)raw;
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
    NSAccessibilityPostNotification(host.view, NSAccessibilityLayoutChangedNotification);
}

@implementation LlrView (LlrA11y)

- (BOOL)isAccessibilityElement {
    return NO;
}

- (NSAccessibilityRole)accessibilityRole {
    return NSAccessibilityGroupRole;
}

- (NSArray *)accessibilityChildren {
    NSArray *els = objc_getAssociatedObject(self, llr_acc_key);
    return els ? els : @[];
}

@end
