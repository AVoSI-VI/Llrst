#!/usr/bin/env bash
# Builds tests/run/hello.llr with --emit-obj for each cross target and checks `file`.
# aarch64, riscv64, and s390x are then linked freestanding with lld. apple-darwin is not.
# Output: ok|cross|TARGET or fail|cross|target=...|got=...
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
llrc="${LLRC:-$root/target/release/llrc}"
[ -x "$llrc" ] || llrc="$(command -v llrc || true)"
if [ -z "$llrc" ] || [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
fi
command -v file >/dev/null || {
  echo "fail|setup|file=missing"
  exit 1
}
if ! command -v ld.lld >/dev/null; then
  for c in "$HOME"/.rustup/toolchains/*/lib/rustlib/*/bin/gcc-ld/ld.lld; do
    if [ -x "$c" ]; then
      PATH="$(dirname "$c"):$PATH"
      break
    fi
  done
fi
command -v ld.lld >/dev/null || {
  echo "fail|setup|lld=missing|fix=install lld"
  exit 1
}
export LLRST_STD="$root/std"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fails=0

check() {
  local triple="$1" needle="$2" obj="$tmp/hello.o"
  rm -f "$obj"
  if ! "$llrc" build "$root/tests/run/hello.llr" --target "$triple" --emit-obj -o "$tmp/hello" 2>"$tmp/err"; then
    echo "fail|cross|target=$triple|got=$(head -1 "$tmp/err")"
    fails=$((fails+1))
    return
  fi
  local got
  got="$(file "$obj")"
  if printf '%s\n' "$got" | grep -q "$needle"; then
    echo "ok|cross|$triple"
  else
    echo "fail|cross|target=$triple|got=$got"
    fails=$((fails+1))
  fi
}

link_elf() {
  local triple="$1" clang_triple="$2" needle="$3" obj="$tmp/hello.o" bin="$tmp/hello.bin"
  check "$triple" "$needle"
  if [ ! -f "$obj" ]; then
    return
  fi
  if ! clang --target="$clang_triple" -c -ffreestanding -fno-stack-protector -fno-builtin \
      "$root/tests/cross_stub.c" -o "$tmp/stub.o" 2>"$tmp/err"; then
    echo "fail|cross|link|target=$triple|got=$(head -1 "$tmp/err")"
    fails=$((fails+1))
    return
  fi
  if ! clang --target="$clang_triple" -fuse-ld=lld -nostdlib -static -Wl,-e,_start \
      "$tmp/stub.o" "$obj" -o "$bin" 2>"$tmp/err"; then
    echo "fail|cross|link|target=$triple|got=$(head -1 "$tmp/err")"
    fails=$((fails+1))
    return
  fi
  local got
  got="$(file "$bin")"
  if printf '%s\n' "$got" | grep -q "executable" && printf '%s\n' "$got" | grep -q "$needle"; then
    echo "ok|cross|link|$triple"
  else
    echo "fail|cross|link|target=$triple|got=$got"
    fails=$((fails+1))
  fi
}

link_elf aarch64-unknown-linux-gnu aarch64-unknown-linux-gnu "ARM aarch64"
link_elf riscv64gc-unknown-linux-gnu riscv64-unknown-linux-gnu "RISC-V"
link_elf s390x-unknown-linux-gnu s390x-unknown-linux-gnu "S/390"
check aarch64-apple-darwin "arm64"

mir="$("$llrc" emit-mir --target x86_64-pc-windows-gnu "$root/tests/run/os_id.llr" 2>"$tmp/err")" || {
  echo "fail|cross|target=x86_64-pc-windows-gnu|got=$(head -1 "$tmp/err")"
  fails=$((fails+1))
  mir=""
}
if [ -n "$mir" ]; then
  main="$(printf '%s\n' "$mir" | awk 'BEGIN{p=0} /^fn main\.main /{p=1; next} /^fn /{p=0} p')"
  if printf '%s\n' "$main" | grep -q '2i64'; then
    echo "ok|cross|x86_64-pc-windows-gnu"
  else
    echo "fail|cross|target=x86_64-pc-windows-gnu|got=os_id"
    fails=$((fails+1))
  fi
fi
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
