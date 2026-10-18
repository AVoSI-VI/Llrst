#!/usr/bin/env bash
# Builds llrc, compiles every tool in tools/, installs binaries and std, verifies.
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
# --emit-obj does not link. A full `llrc build --target TRIPLE` link runs
# `clang --target=TRIPLE` and needs a sysroot for that triple.

say "step|build-llrc"
( cd "$root" && cargo build --release -q ) || die "build-llrc|cargo build --release"
target_dir="$(cd "$root" && cargo metadata --format-version 1 --no-deps 2>/dev/null | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')"
[ -n "$target_dir" ] || target_dir="$root/target"
llrc="$target_dir/release/llrc"
[ -x "$llrc" ] || die "build-llrc|binary=missing|path=$llrc"

mkdir -p "$bin" "$share" || die "install|mkdir=$prefix"
rm -rf "$share/std" && cp -r "$root/std" "$share/std" || die "install|copy=std"
cc -c -fPIC "$root/runtime/llr_stat.c" -o "$share/llr_stat.o" || die "build-runtime|cc llr_stat.c"
say "ok|runtime|$share/llr_stat.o"
cp "$llrc" "$bin/llrc" || die "install|copy=llrc"
say "ok|llrc|$bin/llrc"
say "ok|std|$share/std"

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
printf '(use std.io)\n(fn main () unit (io.puts "hello world"))\n' >"$tmp/hello.llr"
"$bin/llrc" build "$tmp/hello.llr" -o "$tmp/hello" || die "verify|compile hello"
out="$("$tmp/hello")"
[ "$out" = "hello world" ] || die "verify|want=hello world|got=$out"
"$bin/llr-grep" "fn main" "$tmp" >/dev/null || die "verify|llr-grep"
rm -rf "$tmp"
say "ok|verify|hello world"

case ":$PATH:" in
  *":$bin:"*) ;;
  *) say "W|path|$bin is not on PATH|fix=export PATH=\"$bin:\$PATH\"" ;;
esac
say "done|prefix=$prefix|tools=$(ls "$root"/tools/llr-*.llr | wc -l | tr -d ' ')"
