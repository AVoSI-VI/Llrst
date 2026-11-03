# Mobile hosts

Android and iOS present the same BGRA8 buffer as the desktop hosts and pull the same
64-byte event record. The suite stays headless. A windowed program calls the host for
the target it was linked for:

```
(use ui.app)
(use ui.host_android)
(app.run (host_android.host_android.open 0 0) (root) 1000000)
```

`host_ios.host_ios.open` is the iOS side. `open` ignores the requested size and uses the
screen. Logical `size` is the safe content rect, so layout sits inside the status and
navigation bars. The shim translates touches into that rect and blits the bitmap there.
Inset events still report the margins, and `sched.ptrs` stores the last one.

Life events are `life|pause`, `life|resume`, and `life|destroy`. Pause skips layout,
paint, and present. Destroy sets quit. There is no background execution and only one
window.

## Prerequisites

Android needs `ANDROID_NDK` and `ANDROID_HOME` (or `ANDROID_SDK_ROOT`) with a platforms
`android.jar` and a build-tools directory that contains `aapt2`, `zipalign`, and
`apksigner`. `keytool` creates the debug keystore on the first pack. iOS needs Xcode,
so `xcrun` can see the `iphoneos` and `iphonesimulator` SDKs. `self/` does not emit an
`x86_64` Mach-O, so the simulator slice is arm64. An Intel simulator slice is not
produced.

Install compiles the target objects when those SDKs are present. Otherwise it prints
`skip|runtime|android=ndk-missing` or `skip|runtime|ios=sdk-missing` and continues.
`llr-ui pack` compiles the same objects into its out dir when the installed copies are
missing.

## Pack

```
llr-ui pack --target aarch64-linux-android FILE [--out DIR]
llr-ui pack --target aarch64-apple-ios-simulator FILE [--out DIR] [--sign IDENT]
llr-ui pack --target aarch64-apple-ios FILE [--out DIR] --sign IDENT
```

Android success is `pack|apk=PATH`. The APK is a `NativeActivity` with
`android:extractNativeLibs="true"`, `minSdk` 21, and `lib/arm64-v8a/libSTEM.so`.
The `.so` exports `android_main`, which calls the program `main`. A program that
references `llr_ui_open` also links the Android UI shim.

iOS success is `pack|app=PATH`. The bundle is `STEM.app` with `Info.plist`
(`MinimumOSVersion` 14.0, no scene manifest). The link renames `_main` to
`_llr_user_main` and UIKit's `main` calls it. A simulator pack codesigns with `--sign`
or ad-hoc `-` when `--sign` is omitted. A device pack fails `pack|fail|sign=missing`
without an identity. Pack does not install or boot a simulator and does not deploy to
a phone.

```
xcrun simctl install booted PATH.app
xcrun simctl launch booted llr.STEM
```

A missing SDK tool prints `pack|fail|...=missing` and exits 1.
