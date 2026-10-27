#!/usr/bin/env bash
# Diff llrs check stderr against llrc check.
# Usage: tests/check.sh [LLRS]
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
llrc="${LLRC:-$root/target/release/llrc}"
[ -x "$llrc" ] || llrc="$root/target/debug/llrc"
[ -x "$llrc" ] || llrc="$(command -v llrc || true)"
if [ -z "$llrc" ] || [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
fi
export LLRST_STD="$root/std"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
stage="${1:-}"
if [ -z "$stage" ]; then
  if ! "$llrc" build "$root/self/llrs.llr" -o "$tmp/llrs" --std "$root/std" 2>"$tmp/llrs.err"; then
    echo "fail|setup|llrs=$(head -1 "$tmp/llrs.err")"
    exit 1
  fi
  stage="$tmp/llrs"
  [ -x "$stage" ] || stage="$tmp/llrs.exe"
fi
if [ ! -x "$stage" ]; then
  echo "fail|setup|stage=not found|path=$stage"
  exit 1
fi
fails=0
for dir in err warn; do
  for f in "$root"/tests/$dir/*.llr; do
    [ -e "$f" ] || continue
    name="$(basename "$f" .llr)"
    golden="$root/tests/$dir/$name.err"
    [ -e "$golden" ] || continue
    flags=()
    if [ -f "$root/tests/$dir/$name.flags" ]; then
      # shellcheck disable=SC2206
      flags=($(tr -d '\r' <"$root/tests/$dir/$name.flags"))
    fi
    rel="tests/$dir/$name.llr"
    ( cd "$root" && "$llrc" check "$rel" "${flags[@]}" >"$tmp/want.out" 2>"$tmp/want" || true )
    ( cd "$root" && "$stage" check "$rel" "${flags[@]}" >"$tmp/got.out" 2>"$tmp/got" || true )
    tr -d '\r' <"$tmp/want" >"$tmp/want.n"
    tr -d '\r' <"$tmp/got" >"$tmp/got.n"
    if cmp -s "$tmp/want.n" "$tmp/got.n"; then
      echo "ok|check/$rel"
    else
      echo "fail|check/$rel|want=$(head -c 180 "$tmp/want.n" | tr '\n' ' ')|got=$(head -c 180 "$tmp/got.n" | tr '\n' ' ')"
      fails=$((fails+1))
    fi
  done
done
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
