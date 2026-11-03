/* NativeActivity entry. The glue calls android_main; that calls the program main. */
#include "llr_ui_android.h"

struct android_app *llr_android_app;

extern int main(int argc, char **argv);

void android_main(struct android_app *app) {
    char arg0[] = "llr";
    char *argv[2];
    argv[0] = arg0;
    argv[1] = NULL;
    llr_android_app = app;
    main(1, argv);
}
