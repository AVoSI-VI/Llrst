#!/usr/bin/env bash
# Builds stage-0 llrc, bootstraps llrs2, installs that compiler, compiles every tool, verifies.
# Usage: scripts/install.sh [PREFIX]   (default ~/.local)
# Output: one pipe record per step. Exit 1 on failure.
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
prefix="${1:-$HOME/.local}"
bin="$prefix/bin"
share="$prefix/share/llrst"

say() { echo "$*"; }
die() { echo "fail|$*"; exit 1; }

command -v cargo >/dev/null || die "setup|cargo=missing|fix=install rustup"
command -v cc >/dev/null || die "setup|cc=missing|fix=install a C toolchain (used as linker)"

say "step|build-llrc"
( cd "$root" && cargo build --release -q ) || die "build-llrc|cargo build --release"
target_dir="$(cd "$root" && cargo metadata --format-version 1 --no-deps 2>/dev/null | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
[ -n "$target_dir" ] || target_dir="$root/target"
llrc="$target_dir/release/llrc"
[ -x "$llrc" ] || die "build-llrc|binary=missing|path=$llrc"

mkdir -p "$bin" "$share" || die "install|mkdir=$prefix"
rm -rf "$share/std" && cp -r "$root/std" "$share/std" || die "install|copy=std"
rm -rf "$share/ui" && cp -r "$root/ui" "$share/ui" || die "install|copy=ui"
rm -rf "$share/examples" && cp -r "$root/examples" "$share/examples" || die "install|copy=examples"
rm -rf "$share/web" && cp -r "$root/web" "$share/web" || die "install|copy=web"
cc -c -fPIC "$root/runtime/llr_stat.c" -o "$share/llr_stat.o" || die "build-runtime|cc llr_stat.c"
cc -c -fPIC "$root/runtime/llr_now.c" -o "$share/llr_now.o" || die "build-runtime|cc llr_now.c"
cc -c -fPIC "$root/runtime/llr_ui_web.c" -o "$share/llr_ui_web.o" || die "build-runtime|cc llr_ui_web.c"
say "ok|runtime|$share/llr_stat.o"
os_name="$(uname -s)"
if [ "$os_name" = "Darwin" ]; then
  clang -fobjc-arc -c "$root/runtime/llr_ui_mac.m" -o "$share/llr_ui_mac.o" || die "build-runtime|clang llr_ui_mac.m"
  clang -fobjc-arc -c "$root/runtime/llr_ui_mac_in.m" -o "$share/llr_ui_mac_in.o" || die "build-runtime|clang llr_ui_mac_in.m"
  clang -fobjc-arc -c "$root/runtime/llr_ui_mac_a11y.m" -o "$share/llr_ui_mac_a11y.o" || die "build-runtime|clang llr_ui_mac_a11y.m"
  say "ok|runtime|$share/llr_ui_mac.o"
elif [ "$os_name" = "Linux" ]; then
  cc -c -fPIC "$root/runtime/llr_ui_x11.c" -o "$share/llr_ui_x11.o" || die "build-runtime|cc llr_ui_x11.c"
  cc -c -fPIC "$root/runtime/llr_ui_x11_in.c" -o "$share/llr_ui_x11_in.o" || die "build-runtime|cc llr_ui_x11_in.c"
  say "ok|runtime|$share/llr_ui_x11.o"
fi

ndk_tag() {
  local s m
  s="$(uname -s)"
  m="$(uname -m)"
  if [ "$s" = Darwin ]; then
    if [ "$m" = arm64 ] || [ "$m" = aarch64 ]; then echo darwin-arm64; else echo darwin-x86_64; fi
  elif [ "$m" = aarch64 ] || [ "$m" = arm64 ]; then
    echo linux-aarch64
  else
    echo linux-x86_64
  fi
}

compile_android() {
  local ndk="${ANDROID_NDK:-}"
  local tag cc glue inc
  if [ -z "$ndk" ]; then say "skip|runtime|android=ndk-missing"; return; fi
  tag="$(ndk_tag)"
  cc="$ndk/toolchains/llvm/prebuilt/$tag/bin/aarch64-linux-android21-clang"
  if [ ! -x "$cc" ] && [ "$tag" = darwin-arm64 ]; then
    cc="$ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android21-clang"
  fi
  if [ ! -x "$cc" ]; then say "skip|runtime|android=clang-missing"; return; fi
  glue="$ndk/sources/android/native_app_glue"
  inc="-I$root/runtime -I$glue"
  "$cc" -c -fPIC $inc "$root/runtime/llr_stat.c" -o "$share/llr_stat_android.o" || die "build-runtime|cc llr_stat.c|target=android"
  "$cc" -c -fPIC $inc "$root/runtime/llr_now.c" -o "$share/llr_now_android.o" || die "build-runtime|cc llr_now.c|target=android"
  "$cc" -c -fPIC $inc "$root/runtime/llr_android_main.c" -o "$share/llr_android_main.o" || die "build-runtime|cc llr_android_main.c"
  "$cc" -c -fPIC $inc "$root/runtime/llr_ui_android.c" -o "$share/llr_ui_android.o" || die "build-runtime|cc llr_ui_android.c"
  "$cc" -c -fPIC $inc "$root/runtime/llr_ui_android_in.c" -o "$share/llr_ui_android_in.o" || die "build-runtime|cc llr_ui_android_in.c"
  "$cc" -c -fPIC $inc "$root/runtime/llr_ui_android_a11y.c" -o "$share/llr_ui_android_a11y.o" || die "build-runtime|cc llr_ui_android_a11y.c"
  "$cc" -c -fPIC $inc "$glue/android_native_app_glue.c" -o "$share/llr_ui_android_glue.o" || die "build-runtime|cc android_native_app_glue.c"
  say "ok|runtime|$share/llr_ui_android.o"
}

ios_cc() {
  local sdk="$1" tgt="$2" src="$3" out="$4" arc="$5"
  if [ "$arc" = arc ]; then
    xcrun clang -fobjc-arc -c -target "$tgt" -isysroot "$sdk" -I"$root/runtime" "$src" -o "$out" || die "build-runtime|clang $(basename "$src")|out=$out"
  else
    xcrun clang -c -target "$tgt" -isysroot "$sdk" -I"$root/runtime" "$src" -o "$out" || die "build-runtime|clang $(basename "$src")|out=$out"
  fi
}

compile_ios_slice() {
  local sdk="$1" tgt="$2"
  ios_cc "$sdk" "$tgt" "$root/runtime/llr_stat.c" "$share/$3" c
  ios_cc "$sdk" "$tgt" "$root/runtime/llr_now.c" "$share/$4" c
  ios_cc "$sdk" "$tgt" "$root/runtime/llr_ios_main.m" "$share/$5" arc
  ios_cc "$sdk" "$tgt" "$root/runtime/llr_ui_ios.m" "$share/$6" arc
  ios_cc "$sdk" "$tgt" "$root/runtime/llr_ui_ios_in.m" "$share/$7" arc
  ios_cc "$sdk" "$tgt" "$root/runtime/llr_ui_ios_a11y.m" "$share/$8" arc
}

compile_ios() {
  local sim dev
  sim="$(xcrun --sdk iphonesimulator --show-sdk-path 2>/dev/null || true)"
  dev="$(xcrun --sdk iphoneos --show-sdk-path 2>/dev/null || true)"
  if [ -z "$sim" ] || [ -z "$dev" ]; then say "skip|runtime|ios=sdk-missing"; return; fi
  compile_ios_slice "$dev" arm64-apple-ios llr_stat_ios.o llr_now_ios.o llr_ios_main.o llr_ui_ios.o llr_ui_ios_in.o llr_ui_ios_a11y.o
  compile_ios_slice "$sim" arm64-apple-ios-simulator llr_stat_ios_sim.o llr_now_ios_sim.o llr_ios_main_sim.o llr_ui_ios_sim.o llr_ui_ios_sim_in.o llr_ui_ios_sim_a11y.o
  say "ok|runtime|$share/llr_ui_ios.o"
}

compile_android
if [ "$os_name" = Darwin ]; then
  compile_ios
fi
LLRC="$llrc" bash "$root/scripts/self.sh" --stage || exit 1
stage="$root/target/self/llrs2"
[ -f "$stage" ] || stage="$root/target/self/llrs2.exe"
[ -f "$stage" ] || die "install|llrs2=missing|path=$root/target/self/llrs2"
cp "$stage" "$bin/llrc" || die "install|copy=llrs2"
say "ok|llrs2|$bin/llrc"
say "ok|std|$share/std"
say "ok|ui|$share/ui"
say "ok|web|$share/web"

export LLRST_STD="$share/std"
for src in "$root"/tools/llr-*.llr; do
  name="$(basename "$src" .llr)"
  say "step|build-$name"
  if "$bin/llrc" build "$src" -o "$bin/$name" 2>"/tmp/llr_install_$name.err"; then
    say "ok|$name|$bin/$name"
  else
    cat "/tmp/llr_install_$name.err"
    die "build-$name"
  fi
done

say "step|verify"
tmp="$(mktemp -d)"
"$bin/llrc" build "$root/examples/hello.llr" -o "$tmp/hello" || die "verify|compile hello"
out="$("$tmp/hello")"
[ "$out" = "hello world" ] || die "verify|want=hello world|got=$out"
"$bin/llr-grep" "fn main" "$root/examples" >/dev/null || die "verify|llr-grep"
rm -rf "$tmp"
say "ok|verify|hello world"

case ":$PATH:" in
  *":$bin:"*) ;;
  *) say "W|path|$bin is not on PATH|fix=export PATH=\"$bin:\$PATH\"" ;;
esac
say "done|prefix=$prefix|tools=$(ls "$root"/tools/llr-*.llr | wc -l | tr -d ' ')"
