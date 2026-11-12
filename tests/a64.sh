#!/usr/bin/env bash
# tests/run goldens as aarch64-unknown-linux-gnu. A Linux aarch64 host runs the
# binary. Any other host uses qemu-aarch64. A Darwin arm64 host without qemu
# builds each golden as a native Mach-O and runs it directly. Not part of tests/run.sh.
# Output: ok|run/NAME or fail|run/NAME|... or fail|setup|qemu=missing
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
llrc="${LLRC:-$root/target/release/llrc}"
[ -x "$llrc" ] || llrc="$(command -v llrc || true)"
if [ -z "$llrc" ] || [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
fi
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
need_qemu=1
native=0
mach="$(uname -m)"
if [ "$(uname -s)" = Linux ]; then
  if [ "$mach" = aarch64 ] || [ "$mach" = arm64 ]; then
    need_qemu=0
  fi
elif [ "$(uname -s)" = Darwin ] && [ "$mach" = arm64 ] && ! command -v qemu-aarch64 >/dev/null; then
  need_qemu=0
  native=1
  echo "note|a64|native mach-o, qemu-aarch64 missing"
fi
if [ "$need_qemu" -eq 1 ] && ! command -v qemu-aarch64 >/dev/null; then
  echo "fail|setup|qemu=missing"
  exit 1
fi

host_cc() {
  if [ -n "${LLR_CC:-}" ]; then
    printf '%s\n' "$LLR_CC"
  elif [ -x /opt/homebrew/opt/llvm/bin/clang ]; then
    printf '%s\n' /opt/homebrew/opt/llvm/bin/clang
  elif [ -x /usr/local/opt/llvm/bin/clang ]; then
    printf '%s\n' /usr/local/opt/llvm/bin/clang
  else
    printf '%s\n' clang
  fi
}

export LLRST_STD="$root/std"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fails=0
cc="$(host_cc)"
if [ "$native" -eq 0 ] && ! "$cc" --target=aarch64-unknown-linux-gnu -c -ffreestanding -fno-stack-protector -fno-builtin \
    "$root/tests/a64_stub.c" -o "$tmp/stub.o" 2>"$tmp/stub.err"; then
  echo "fail|setup|cc=$(head -1 "$tmp/stub.err")"
  exit 1
fi
if [ "$native" -eq 1 ]; then
  export LLRST_RUNTIME="$root/runtime"
  for c in llr_stat llr_now llr_reload llr_dl; do
    cc -c -fPIC "$root/runtime/$c.c" -o "$root/runtime/$c.o" 2>/dev/null || {
      echo "fail|setup|cc=$c.c"
      exit 1
    }
  done
fi

run_bin() {
  if [ "$need_qemu" -eq 1 ]; then
    qemu-aarch64 "$1"
  else
    "$1"
  fi
}

check_one() {
  local name="$1" code="$2"
  local want="$root/tests/run/$name.out"
  local ok=1
  if ! cmp -s "$tmp/$name.out" "$want"; then
    echo "fail|run/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.out" | tr '\n' ' ')"
    fails=$((fails + 1))
    ok=0
  fi
  if [ -f "$root/tests/run/$name.err" ] && ! cmp -s "$tmp/$name.err" "$root/tests/run/$name.err"; then
    echo "fail|run/$name|stderr|want=$(head -c 200 "$root/tests/run/$name.err" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.err" | tr '\n' ' ')"
    fails=$((fails + 1))
    ok=0
  elif [ ! -f "$root/tests/run/$name.err" ] && [ -s "$tmp/$name.err" ]; then
    echo "fail|run/$name|stderr|got=$(head -c 200 "$tmp/$name.err" | tr '\n' ' ')"
    fails=$((fails + 1))
    ok=0
  fi
  if [ -f "$root/tests/run/$name.code" ]; then
    if ! cmp -s "$tmp/$name.code" "$root/tests/run/$name.code"; then
      echo "fail|run/$name|code|want=$(head -c 200 "$root/tests/run/$name.code" | tr '\n' ' ')|got=$code"
      fails=$((fails + 1))
      ok=0
    fi
  elif [ "$code" -ne 0 ]; then
    echo "fail|run/$name|code=$code"
    fails=$((fails + 1))
    ok=0
  fi
  if [ "$ok" -eq 1 ]; then
    echo "ok|run/$name"
  fi
}

for f in "$root"/tests/run/*.llr; do
  name="$(basename "$f" .llr)"
  [ -e "${f%.llr}.out" ] || continue
  flags=()
  if [ -f "${f%.llr}.flags" ]; then
    read -r -a flags <"${f%.llr}.flags"
  fi
  set +u
  if [ "$native" -eq 1 ]; then
    "$llrc" build "$f" "${flags[@]}" -o "$tmp/$name.bin" 2>"$tmp/$name.cerr"
  else
    "$llrc" build "$f" --emit-obj --target aarch64-unknown-linux-gnu "${flags[@]}" -o "$tmp/$name" 2>"$tmp/$name.cerr"
  fi
  built=$?
  set -u
  if [ "$built" -ne 0 ]; then
    echo "fail|run/$name|compile|$(head -1 "$tmp/$name.cerr")"
    fails=$((fails + 1))
    continue
  fi
  if [ "$native" -eq 0 ] && ! "$cc" --target=aarch64-unknown-linux-gnu -fuse-ld=lld -nostdlib -static -Wl,-e,_start \
      "$tmp/stub.o" "$tmp/$name.o" -o "$tmp/$name.bin" 2>"$tmp/$name.lerr"; then
    echo "fail|run/$name|link|$(head -1 "$tmp/$name.lerr")"
    fails=$((fails + 1))
    continue
  fi
  run_bin "$tmp/$name.bin" >"$tmp/$name.out" 2>"$tmp/$name.err"
  code=$?
  printf '%s\n' "$code" >"$tmp/$name.code"
  check_one "$name" "$code"
done
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
