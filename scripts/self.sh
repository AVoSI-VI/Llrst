#!/usr/bin/env bash
# Stage bootstrap. llrc builds llrs0, llrs0 builds llrs1, llrs1 builds llrs2.
# Every stage is linked as the same path and then copied, so the output file
# name is not part of the image. cmp must be byte-equal, then the run suite uses llrs2.
# Usage: scripts/self.sh
# Output: one pipe record per step. Exit 1 on failure.
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root" || exit 1

pick_exe() {
  if [ -n "$1" ] && [ -x "$1" ]; then
    printf '%s\n' "$1"
  elif [ -n "$1" ] && [ -x "$1.exe" ]; then
    printf '%s\n' "$1.exe"
  else
    printf '\n'
  fi
}

llrc="${LLRC:-$root/target/release/llrc}"
got="$(pick_exe "$llrc")"
if [ -z "$got" ]; then
  got="$(pick_exe "$root/target/debug/llrc")"
fi
if [ -z "$got" ]; then
  got="$(command -v llrc || true)"
fi
llrc="$got"
if [ -z "$llrc" ] || [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
fi
command -v cc >/dev/null || {
  echo "fail|setup|cc=missing|fix=install a C toolchain (used as linker)"
  exit 1
}

export LLRST_STD="$root/std"
export LLRST_RUNTIME="$root/runtime"
# A linker clock must not split llrs1 from llrs2. The suite does not need it.
export SOURCE_DATE_EPOCH=0
cc -c -fPIC "$root/runtime/llr_stat.c" -o "$root/runtime/llr_stat.o" || {
  echo "fail|setup|cc=llr_stat.c"
  exit 1
}

out="$root/target/self"
work="$out/work"
mkdir -p "$work" || {
  echo "fail|setup|mkdir=$work"
  exit 1
}
rm -f "$out/llrs0" "$out/llrs0.exe" "$out/llrs1" "$out/llrs1.exe" "$out/llrs2" "$out/llrs2.exe"

stage() {
  name="$1"
  comp="$2"
  echo "step|$name"
  rm -f "$work/llrs" "$work/llrs.exe" "$work/llrs.o" "$work/llrs.obj"
  if ! "$comp" build "$root/self/llrs.llr" -o "$work/llrs" --std "$root/std" 2>"$out/$name.err"; then
    msg="$(head -1 "$out/$name.err" 2>/dev/null || true)"
    [ -n "$msg" ] || msg="build failed"
    echo "fail|$name|$msg"
    return 1
  fi
  src="$work/llrs"
  [ -f "$src" ] || src="$work/llrs.exe"
  if [ ! -f "$src" ]; then
    echo "fail|$name|binary=missing|path=$work/llrs"
    return 1
  fi
  cp -f "$src" "$out/$name" || {
    echo "fail|$name|copy=$out/$name"
    return 1
  }
  chmod +x "$out/$name" 2>/dev/null || true
  echo "ok|$name|$out/$name"
}

stage llrs0 "$llrc" || exit 1
stage llrs1 "$out/llrs0" || exit 1
stage llrs2 "$out/llrs1" || exit 1

if cmp -s "$out/llrs1" "$out/llrs2"; then
  echo "self|stage2=equal"
else
  echo "fail|stage2|llrs1!=llrs2"
  exit 1
fi

unset SOURCE_DATE_EPOCH
LLRC="$out/llrs2" "$root/tests/run.sh"
