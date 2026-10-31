#!/usr/bin/env bash
# Snapshot of every x86-64 object self/ emits. --save DIR writes them.
# The default compares against target/self/x64_ref with cmp.
# LLRC defaults to target/self/llrs0 and is rebuilt when self/*.llr is newer.
# An explicit LLRC is used as-is.
# Output: ok|x64/elf/NAME or fail|x64/coff/NAME|first=<offset>. Last line done|fails=N.
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root" || exit 1
save=""
if [ "${1:-}" = "--save" ]; then
  if [ -z "${2:-}" ]; then
    echo "fail|setup|arg=--save|fix=tests/x64_ref.sh [--save DIR]"
    exit 1
  fi
  save="$2"
elif [ -n "${1:-}" ]; then
  echo "fail|setup|arg=$1|fix=tests/x64_ref.sh [--save DIR]"
  exit 1
fi

ref="$root/target/self/x64_ref"
log="$(mktemp -d)"
if [ -n "$save" ]; then
  out="$save"
  mkdir -p "$out" || {
    echo "fail|setup|mkdir=$out"
    exit 1
  }
  trap 'rm -rf "$log"' EXIT
else
  out="$(mktemp -d)"
  trap 'rm -rf "$out" "$log"' EXIT
fi

pick_exe() {
  if [ -n "$1" ] && [ -x "$1" ]; then
    printf '%s\n' "$1"
  elif [ -n "$1" ] && [ -x "$1.exe" ]; then
    printf '%s\n' "$1.exe"
  else
    printf '\n'
  fi
}

if [ -n "${LLRC:-}" ]; then
  llrc="$(pick_exe "$LLRC")"
  if [ -z "$llrc" ]; then
    echo "fail|setup|llrc=not found|path=$LLRC"
    exit 1
  fi
else
  llrc="$(pick_exe "$root/target/self/llrs0")"
  rebuild=0
  if [ -z "$llrc" ]; then
    rebuild=1
  else
    for f in "$root"/self/*.llr; do
      if [ "$f" -nt "$llrc" ]; then
        rebuild=1
        break
      fi
    done
  fi
  if [ "$rebuild" -eq 1 ]; then
    "$root/scripts/self.sh" --llrs0 || exit 1
    llrc="$(pick_exe "$root/target/self/llrs0")"
  fi
  if [ -z "$llrc" ]; then
    echo "fail|setup|llrc=not found|path=$root/target/self/llrs0"
    exit 1
  fi
fi

export LLRST_STD="$root/std"
fails=0

emit() {
  kind="$1"
  name="$2"
  src="$3"
  testmode="$4"
  if [ "$kind" = "elf" ]; then
    triple="x86_64-unknown-linux-gnu"
    ext=".o"
  else
    triple="x86_64-pc-windows-gnu"
    ext=".obj"
  fi
  base="$out/$kind/$name"
  mkdir -p "$(dirname "$base")" || {
    echo "fail|x64/$kind/$name|mkdir"
    fails=$((fails + 1))
    return
  }
  if [ "$testmode" -eq 1 ]; then
    set +u
    "$llrc" build "$src" --emit-obj --target "$triple" --std "$root/std" --test -o "$base" 2>"$log/err"
    built=$?
    set -u
  else
    "$llrc" build "$src" --emit-obj --target "$triple" --std "$root/std" -o "$base" 2>"$log/err"
    built=$?
  fi
  if [ "$built" -ne 0 ]; then
    echo "fail|x64/$kind/$name|$(head -1 "$log/err")"
    fails=$((fails + 1))
    return
  fi
  got="$base$ext"
  if [ ! -f "$got" ]; then
    echo "fail|x64/$kind/$name|object=missing|path=$got"
    fails=$((fails + 1))
    return
  fi
  if [ -n "$save" ]; then
    echo "ok|x64/$kind/$name"
    return
  fi
  want="$ref/$kind/$name$ext"
  if [ ! -f "$want" ]; then
    echo "fail|x64/$kind/$name|missing"
    fails=$((fails + 1))
    return
  fi
  if cmp -s "$got" "$want"; then
    echo "ok|x64/$kind/$name"
    return
  fi
  line="$(cmp -l "$got" "$want" 2>/dev/null | head -1 || true)"
  # cmp -l pads the offset, so the first field is not at column 0.
  trim="${line#"${line%%[![:space:]]*}"}"
  off="${trim%% *}"
  [ -n "$off" ] || off="eof"
  echo "fail|x64/$kind/$name|first=$off"
  fails=$((fails + 1))
}

for f in "$root"/tests/run/*.llr; do
  grep -q '^(fn main' "$f" || continue
  name="run/$(basename "$f" .llr)"
  emit elf "$name" "$f" 0
  emit coff "$name" "$f" 0
done

for f in "$root"/std/*.llr; do
  grep -q '^(test ' "$f" || continue
  name="std/$(basename "$f" .llr)"
  emit elf "$name" "$f" 1
  emit coff "$name" "$f" 1
done

# The compiler image records source paths as given. The snapshot used a relative path.
emit elf "self/llrs" "self/llrs.llr" 0
emit coff "self/llrs" "self/llrs.llr" 0

echo "done|fails=$fails"
[ "$fails" -eq 0 ]
