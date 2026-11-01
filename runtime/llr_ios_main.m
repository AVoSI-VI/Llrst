/* UIKit owns main. The user program is _llr_user_main after the link renames it. */
#import "llr_ui.h"

#import <UIKit/UIKit.h>
#import <stdlib.h>

extern int llr_user_main(int argc, char **argv);

static int g_argc;
static char **g_argv;
static void (*g_life)(int);

void llr_ios_on_life(void (*fn)(int)) {
    g_life = fn;
}

static void life(int kind) {
    if (g_life) {
        g_life(kind);
    }
}

@interface LlrApp : UIResponder <UIApplicationDelegate>
@end

@implementation LlrApp

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)opts {
    (void)application;
    (void)opts;
    llr_user_main(g_argc, g_argv);
    exit(0);
    return YES;
}

- (void)applicationDidEnterBackground:(UIApplication *)application {
    (void)application;
    life(LLR_UI_PAUSE);
}

- (void)applicationWillEnterForeground:(UIApplication *)application {
    (void)application;
    life(LLR_UI_RESUME);
}

- (void)applicationWillTerminate:(UIApplication *)application {
    (void)application;
    life(LLR_UI_DESTROY);
}

@end

int main(int argc, char **argv) {
    g_argc = argc;
    g_argv = argv;
    @autoreleasepool {
        return UIApplicationMain(argc, argv, nil, NSStringFromClass([LlrApp class]));
    }
}
