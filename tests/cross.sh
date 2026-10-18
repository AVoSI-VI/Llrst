#!/usr/bin/env bash
# Builds tests/run/hello.llr with --emit-obj for each cross target and checks `file`.
# Does not link. Output: ok|cross|TARGET or fail|cross|target=...|got=...
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

check aarch64-unknown-linux-gnu "ARM aarch64"
check riscv64gc-unknown-linux-gnu "RISC-V"
check s390x-unknown-linux-gnu "S/390"
check aarch64-apple-darwin "arm64"
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
