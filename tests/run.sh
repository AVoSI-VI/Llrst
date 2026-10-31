#!/usr/bin/env bash
# Golden tests: tests/run/*.llr (stdout must equal .out; stderr equals .err when that
# file exists, and must be empty otherwise; exit status is 0 unless .code exists),
# tests/ui/*.llr (same, built with --std std --ui ui; NAME.ppm is compared to --shot output),
# tests/err/*.llr (stderr must equal .err),
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
  if [ "$name" = os_id ] && [ "${OS:-}" = "Windows_NT" ]; then
    printf '2\n' >"$tmp/os_id.want"
    want="$tmp/os_id.want"
  fi
  flags=()
  if [ -f "${f%.llr}.flags" ]; then
    read -r -a flags <"${f%.llr}.flags"
  fi
  # bash 3.2 (macOS) errors on an empty "${flags[@]}" under set -u.
  set +u
  "$llrc" build "$f" "${flags[@]}" -o "$tmp/$name" 2>"$tmp/$name.cerr"
  built=$?
  set -u
  if [ "$built" -ne 0 ]; then
    echo "fail|run/$name|compile|$(head -1 "$tmp/$name.cerr")"; fails=$((fails+1)); continue
  fi
  "$tmp/$name" >"$tmp/$name.out" 2>"$tmp/$name.stderr"
  code=$?
  printf '%s\n' "$code" >"$tmp/$name.code"
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
  elif [ ! -f "${f%.llr}.err" ] && [ -s "$tmp/$name.stderr" ]; then
    echo "fail|run/$name|stderr|got=$(head -c 200 "$tmp/$name.stderr" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ -f "${f%.llr}.code" ]; then
    if ! cmp -s "$tmp/$name.code" "${f%.llr}.code"; then
      echo "fail|run/$name|code|want=$(head -c 200 "${f%.llr}.code" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.code" | tr '\n' ' ')"
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
done

for f in "$root"/tests/ui/*.llr; do
  [ -e "$f" ] || continue
  name="$(basename "$f" .llr)"
  want="${f%.llr}.out"
  [ -e "$want" ] || continue
  set +u
  ( cd "$root" && "$llrc" build "tests/ui/$name.llr" --std std --ui ui -o "$tmp/ui_$name" 2>"$tmp/ui_$name.cerr" )
  built=$?
  set -u
  if [ "$built" -ne 0 ]; then
    echo "fail|ui/$name|compile|$(head -1 "$tmp/ui_$name.cerr")"; fails=$((fails+1)); continue
  fi
  shot="$tmp/ui_$name.shot"
  args=()
  if [ -f "${f%.llr}.ppm" ]; then
    args=(--shot "$shot")
  fi
  set +u
  "$tmp/ui_$name" "${args[@]}" >"$tmp/ui_$name.out" 2>"$tmp/ui_$name.stderr"
  code=$?
  set -u
  ok=1
  if ! cmp -s "$tmp/ui_$name.out" "$want"; then
    echo "fail|ui/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/ui_$name.out" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ -s "$tmp/ui_$name.stderr" ]; then
    echo "fail|ui/$name|stderr|got=$(head -c 200 "$tmp/ui_$name.stderr" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ "$code" -ne 0 ]; then
    echo "fail|ui/$name|code=$code"
    fails=$((fails+1))
    ok=0
  fi
  if [ -f "${f%.llr}.ppm" ] && ! cmp -s "$shot" "${f%.llr}.ppm"; then
    echo "fail|ui/$name|ppm|want=$(head -c 200 "${f%.llr}.ppm" | tr '\n' ' ')|got=$(head -c 200 "$shot" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ "$ok" -eq 1 ]; then
    echo "ok|ui/$name"
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
  if ! grep -Eq '\|(borrow|move|init)\.|name\.dup|name\.not_module|name\.not_type|flow\.brk_outside|io\.read' "$want"; then
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
    set +u
    flags+=("${extra[@]}")
    set -u
  fi
  set +u
  ( cd "$root" && "$llrc" build "tests/panic/$name.llr" "${flags[@]}" -o "$tmp/panic_$name" 2>"$tmp/panic_$name.cerr" )
  built=$?
  set -u
  if [ "$built" -ne 0 ]; then
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
  flags=()
  if [ -f "${f%.llr}.flags" ]; then
    read -r -a flags <"${f%.llr}.flags"
  fi
  set +u
  ( cd "$root" && "$llrc" check "tests/warn/$name.llr" "${flags[@]}" 2>"$tmp/warn_$name.err" >/dev/null )
  code=$?
  set -u
  printf '%s\n' "$code" >"$tmp/warn_$name.code"
  ok=1
  if ! cmp -s "$tmp/warn_$name.err" "$want"; then
    echo "fail|warn/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/warn_$name.err" | tr '\n' ' ')"
    fails=$((fails+1))
    ok=0
  fi
  if [ -f "${f%.llr}.code" ]; then
    if ! cmp -s "$tmp/warn_$name.code" "${f%.llr}.code"; then
      echo "fail|warn/$name|code=$code|want=$(cat "${f%.llr}.code")"
      fails=$((fails+1))
      ok=0
    fi
  elif [ "$code" -ne 0 ]; then
    echo "fail|warn/$name|code=$code"
    fails=$((fails+1))
    ok=0
  fi
  if [ "$ok" -eq 1 ]; then
    echo "ok|warn/$name"
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

for f in "$root"/ui/*.llr; do
  name="$(basename "$f" .llr)"
  grep -q "^(test " "$f" || continue
  if ! "$llrc" build "$f" --test --std std --ui ui -o "$tmp/uimod_$name" 2>"$tmp/uimod_$name.err"; then
    echo "fail|ui-mod/$name|compile|$(head -1 "$tmp/uimod_$name.err")"; fails=$((fails+1)); continue
  fi
  if "$tmp/uimod_$name" >"$tmp/uimod_$name.out" 2>&1; then
    echo "ok|ui-mod/$name"
  else
    echo "fail|ui-mod/$name|$(grep -E '^(fail|F)\|' "$tmp/uimod_$name.out" | head -3 | tr '\n' ' ')"
    fails=$((fails+1))
  fi
done

"$root/scripts/check-lines.sh" || fails=$((fails+1))

pick_exe() {
  if [ -n "$1" ] && [ -x "$1" ]; then
    printf '%s\n' "$1"
  elif [ -n "$1" ] && [ -x "$1.exe" ]; then
    printf '%s\n' "$1.exe"
  else
    printf '\n'
  fi
}
stage0="$(pick_exe "$root/target/release/llrc")"
[ -n "$stage0" ] || stage0="$(pick_exe "$root/target/debug/llrc")"
if [ -z "$stage0" ] || [ ! -x "$stage0" ]; then
  echo "fail|setup|stage0=not found|fix=cargo build --release"
  fails=$((fails+1))
else
  if [ "$llrc" = "$stage0" ]; then
    LLRC="$stage0" "$root/tests/check.sh" || fails=$((fails+1))
    LLRC="$stage0" "$root/tests/mir.sh" || fails=$((fails+1))
  else
    LLRC="$stage0" "$root/tests/check.sh" "$llrc" || fails=$((fails+1))
    LLRC="$stage0" "$root/tests/mir.sh" "$llrc" || fails=$((fails+1))
  fi
  LLRC="$stage0" "$root/tests/self.sh" || fails=$((fails+1))
  LLRC="$stage0" "$root/tests/cross.sh" || fails=$((fails+1))
  if [ "$llrc" != "$stage0" ]; then
    LLRC="$llrc" "$root/tests/cross.sh" || fails=$((fails+1))
  fi
  if [ -d "$root/target/self/x64_ref" ]; then
    ( unset LLRC; "$root/tests/x64_ref.sh" ) || fails=$((fails+1))
  fi
fi
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
