#!/usr/bin/env bash
# Golden tests: tests/run/*.llr (stdout must equal .out; stderr equals .err and the exit
# status equals .code when those files exist), tests/err/*.llr (stderr must equal .err),
# tests/panic/*.llr (stderr must equal .err and exit 101), tests/warn/*.llr (stderr must equal
# .err and exit 0), and every std/*.llr compiled with --test. Output: ok|NAME or fail|NAME|...
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
llrc="${LLRC:-$root/target/release/llrc}"
[ -x "$llrc" ] || llrc="$(command -v llrc || true)"
if [ -z "$llrc" ] || [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
fi
export LLRST_STD="$root/std"
cc -c -fPIC "$root/runtime/llr_stat.c" -o "$root/runtime/llr_stat.o" || {
  echo "fail|setup|cc=llr_stat.c"
  exit 1
}
export LLRST_RUNTIME="$root/runtime"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fails=0

for f in "$root"/tests/run/*.llr; do
  name="$(basename "$f" .llr)"
  want="${f%.llr}.out"
  [ -e "$want" ] || continue   # helper module for a proj.* test
  flags=()
  if [ -f "${f%.llr}.flags" ]; then
    read -r -a flags <"${f%.llr}.flags"
  fi
  if ! "$llrc" build "$f" "${flags[@]}" -o "$tmp/$name" 2>"$tmp/$name.cerr"; then
    echo "fail|run/$name|compile|$(head -1 "$tmp/$name.cerr")"; fails=$((fails+1)); continue
  fi
  "$tmp/$name" >"$tmp/$name.out" 2>"$tmp/$name.stderr"
  printf '%s\n' "$?" >"$tmp/$name.code"
  if [ "$name" = inline ]; then
    if "$llrc" emit-mir "$f" 2>/dev/null | grep -E '^fn (str\.at|vec\.get)( |$)' >/dev/null; then
      echo "fail|run/$name|mir=str.at or vec.get was not substituted"
      fails=$((fails+1))
    fi
  fi
  ok=1
  if ! cmp -s "$tmp/$name.out" "$want"; then
    echo "fail|run/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.out" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ -f "${f%.llr}.err" ] && ! cmp -s "$tmp/$name.stderr" "${f%.llr}.err"; then
    echo "fail|run/$name|stderr|want=$(head -c 200 "${f%.llr}.err" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.stderr" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ -f "${f%.llr}.code" ] && ! cmp -s "$tmp/$name.code" "${f%.llr}.code"; then
    echo "fail|run/$name|code|want=$(head -c 200 "${f%.llr}.code" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.code" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ "$ok" -eq 1 ]; then
    echo "ok|run/$name"
  fi
done

"$llrc" build "$root/tools/llr-check.llr" -o "$tmp/llr-check" 2>"$tmp/llr-check.err" || {
  echo "fail|setup|llr-check=$(head -1 "$tmp/llr-check.err")"
  exit 1
}

for f in "$root"/tests/err/*.llr; do
  [ -e "$f" ] || continue
  name="$(basename "$f" .llr)"
  want="${f%.llr}.err"
  [ -e "$want" ] || continue   # helper module for a proj.* test
  ( cd "$root" && "$llrc" check "tests/err/$name.llr" 2>"$tmp/$name.err" >/dev/null )
  if cmp -s "$tmp/$name.err" "$want"; then
    echo "ok|err/$name"
  else
    echo "fail|err/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.err" | tr '\n' ' ')"
    fails=$((fails+1))
  fi
  if grep -Eq '\|(type|name)\.' "$want"; then
    ( cd "$root" && "$tmp/llr-check" "tests/err/$name.llr" >/dev/null 2>"$tmp/$name.lerr" || true )
    if cmp -s "$tmp/$name.lerr" "$want"; then
      echo "ok|err/$name|llr-check"
    else
      echo "fail|err/$name|llr-check|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.lerr" | tr '\n' ' ')"
      fails=$((fails+1))
    fi
  fi
done

for f in "$root"/tests/panic/*.llr; do
  [ -e "$f" ] || continue
  name="$(basename "$f" .llr)"
  want="${f%.llr}.err"
  [ -e "$want" ] || continue   # helper module
  flags=(--std std)
  if [ -f "${f%.llr}.flags" ]; then
    read -r -a extra <"${f%.llr}.flags"
    flags+=("${extra[@]}")
  fi
  if ! ( cd "$root" && "$llrc" build "tests/panic/$name.llr" "${flags[@]}" -o "$tmp/panic_$name" 2>"$tmp/panic_$name.cerr" ); then
    echo "fail|panic/$name|compile|$(head -1 "$tmp/panic_$name.cerr")"; fails=$((fails+1)); continue
  fi
  "$tmp/panic_$name" >/dev/null 2>"$tmp/panic_$name.err"; code=$?
  if [ "$code" -eq 101 ] && cmp -s "$tmp/panic_$name.err" "$want"; then
    echo "ok|panic/$name"
  else
    echo "fail|panic/$name|code=$code|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/panic_$name.err" | tr '\n' ' ')"
    fails=$((fails+1))
  fi
done

for f in "$root"/tests/warn/*.llr; do
  [ -e "$f" ] || continue
  name="$(basename "$f" .llr)"
  want="${f%.llr}.err"
  [ -e "$want" ] || continue
  ( cd "$root" && "$llrc" check "tests/warn/$name.llr" 2>"$tmp/warn_$name.err" >/dev/null )
  code=$?
  if [ "$code" -eq 0 ] && cmp -s "$tmp/warn_$name.err" "$want"; then
    echo "ok|warn/$name"
  else
    echo "fail|warn/$name|code=$code|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/warn_$name.err" | tr '\n' ' ')"
    fails=$((fails+1))
  fi
done

for f in "$root"/std/*.llr; do
  name="$(basename "$f" .llr)"
  grep -q "^(test " "$f" || continue
  if ! "$llrc" build "$f" --test -o "$tmp/std_$name" 2>"$tmp/std_$name.err"; then
    echo "fail|std/$name|compile|$(head -1 "$tmp/std_$name.err")"; fails=$((fails+1)); continue
  fi
  if "$tmp/std_$name" >"$tmp/std_$name.out" 2>&1; then
    echo "ok|std/$name"
  else
    echo "fail|std/$name|$(grep -E '^(fail|F)\|' "$tmp/std_$name.out" | head -3 | tr '\n' ' ')"
    fails=$((fails+1))
  fi
done

"$root/scripts/check-lines.sh" || fails=$((fails+1))
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
