#!/usr/bin/env bash
# Diff a stage compiler against llrc check on every tests/err and tests/warn golden.
# Usage: tests/self.sh [STAGE]   (STAGE or the first argument; default builds llr-check)
# Output: ok|self/DIR/NAME or fail|self/DIR/NAME|want=...|got=...
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
stage="${1:-${STAGE:-}}"
if [ -z "$stage" ]; then
  if ! "$llrc" build "$root/tools/llr-check.llr" -o "$tmp/llr-check" 2>"$tmp/llr-check.err"; then
    echo "fail|setup|llr-check=$(head -1 "$tmp/llr-check.err")"
    exit 1
  fi
  stage="$tmp/llr-check"
  [ -x "$stage" ] || stage="$tmp/llr-check.exe"
fi
if [ ! -x "$stage" ]; then
  echo "fail|setup|stage=not found|path=$stage"
  exit 1
fi
fails=0
for dir in err warn; do
  for f in "$root"/tests/"$dir"/*.llr; do
    [ -e "$f" ] || continue
    name="$(basename "$f" .llr)"
    [ -e "${f%.llr}.err" ] || continue
    flags=()
    if [ -f "${f%.llr}.flags" ]; then
      read -r -a flags < <(tr -d '\r' <"${f%.llr}.flags")
    fi
    rel="tests/$dir/$name.llr"
    set +u
    ( cd "$root" && "$llrc" check "$rel" "${flags[@]}" >/dev/null 2>"$tmp/want" || true )
    ( cd "$root" && "$stage" "$rel" "${flags[@]}" >/dev/null 2>"$tmp/got" || true )
    set -u
    tr -d '\r' <"$tmp/want" >"$tmp/want.n"
    tr -d '\r' <"$tmp/got" >"$tmp/got.n"
    if cmp -s "$tmp/want.n" "$tmp/got.n"; then
      echo "ok|self/$dir/$name"
    else
      echo "fail|self/$dir/$name|want=$(head -c 200 "$tmp/want.n" | tr '\n' ' ')|got=$(head -c 200 "$tmp/got.n" | tr '\n' ' ')"
      fails=$((fails+1))
    fi
  done
done
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
