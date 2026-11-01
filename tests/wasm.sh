#!/usr/bin/env bash
# tests/run and tests/panic goldens under wasmtime. Not part of tests/run.sh:
# stage 0 cannot emit wasm, and the host suite stays green without wasmtime.
# Output: ok|NAME or fail|NAME|... and one skip line per unsupported area.
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
llrc="${LLRC:-$root/target/release/llrc}"
[ -x "$llrc" ] || llrc="$(command -v llrc || true)"
if [ -z "$llrc" ] || [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
fi
if ! command -v wasmtime >/dev/null; then
  echo "fail|setup|wasmtime=missing"
  exit 1
fi
export LLRST_STD="$root/std"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fails=0

run_one() {
  local kind="$1" name="$2" wasm="$3"
  wasmtime run "$wasm" >"$tmp/$name.out" 2>"$tmp/$name.err"
  local code=$?
  printf '%s\n' "$code" >"$tmp/$name.code"
  if [ "$kind" = panic ]; then
    local want="$root/tests/panic/$name.err"
    if [ "$code" -eq 101 ] && cmp -s "$tmp/$name.err" "$want"; then
      echo "ok|panic/$name"
    else
      echo "fail|panic/$name|code=$code|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.err" | tr '\n' ' ')"
      fails=$((fails+1))
    fi
    return
  fi
  local want="$root/tests/run/$name.out"
  if [ "$name" = os_id ]; then
    printf '3\n' >"$tmp/os_id.want"
    want="$tmp/os_id.want"
  fi
  local ok=1
  if ! cmp -s "$tmp/$name.out" "$want"; then
    echo "fail|run/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.out" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ -f "$root/tests/run/$name.err" ] && ! cmp -s "$tmp/$name.err" "$root/tests/run/$name.err"; then
    echo "fail|run/$name|stderr|want=$(head -c 200 "$root/tests/run/$name.err" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.err" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  elif [ ! -f "$root/tests/run/$name.err" ] && [ -s "$tmp/$name.err" ]; then
    echo "fail|run/$name|stderr|got=$(head -c 200 "$tmp/$name.err" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ -f "$root/tests/run/$name.code" ]; then
    if ! cmp -s "$tmp/$name.code" "$root/tests/run/$name.code"; then
      echo "fail|run/$name|code|want=$(cat "$root/tests/run/$name.code")|got=$code"
      fails=$((fails+1))
      ok=0
    fi
  elif [ "$code" -ne 0 ]; then
    echo "fail|run/$name|code=$code"
    fails=$((fails+1))
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
  "$llrc" build "$f" --std std --target wasm32-wasi "${flags[@]}" -o "$tmp/$name.wasm" 2>"$tmp/$name.cerr"
  built=$?
  set -u
  if [ "$built" -ne 0 ]; then
    echo "fail|run/$name|compile|$(head -1 "$tmp/$name.cerr")"
    fails=$((fails+1))
    continue
  fi
  run_one run "$name" "$tmp/$name.wasm"
done

for f in "$root"/tests/panic/*.llr; do
  [ -e "$f" ] || continue
  name="$(basename "$f" .llr)"
  [ -e "${f%.llr}.err" ] || continue
  flags=(--std std --target wasm32-wasi)
  if [ -f "${f%.llr}.flags" ]; then
    read -r -a extra <"${f%.llr}.flags"
    set +u
    flags+=("${extra[@]}")
    set -u
  fi
  set +u
  ( cd "$root" && "$llrc" build "tests/panic/$name.llr" "${flags[@]}" -o "$tmp/panic_$name.wasm" 2>"$tmp/panic_$name.cerr" )
  built=$?
  set -u
  if [ "$built" -ne 0 ]; then
    echo "fail|panic/$name|compile|$(head -1 "$tmp/panic_$name.cerr")"
    fails=$((fails+1))
    continue
  fi
  run_one panic "$name" "$tmp/panic_$name.wasm"
done

echo "skip|net|reason=target.unsupported"
echo "skip|thread|reason=target.unsupported"
echo "skip|os.shell|reason=target.unsupported"
echo "skip|fs.list|reason=target.unsupported"
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
