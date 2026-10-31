#!/usr/bin/env bash
# Diff llrs emit-mir against llrc emit-mir.
# Usage: tests/mir.sh [LLRS]
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
for dir in run std; do
  for f in "$root"/$dir/*.llr "$root"/tests/$dir/*.llr; do
    [ -e "$f" ] || continue
    case "$f" in
      "$root"/std/*.llr|"$root"/tests/run/*.llr) ;;
      *) continue ;;
    esac
    rel="${f#$root/}"
    name="$(basename "$f" .llr)"
    set +u
    ( cd "$root" && "$llrc" emit-mir "$rel" --std "$root/std" >"$tmp/want" 2>"$tmp/want.err" || true )
    ( cd "$root" && LLRST_STD="$root/std" "$stage" emit-mir "$rel" >"$tmp/got" 2>"$tmp/got.err" || true )
    set -u
    tr -d '\r' <"$tmp/want" >"$tmp/want.n"
    tr -d '\r' <"$tmp/got" >"$tmp/got.n"
    tr -d '\r' <"$tmp/want.err" >"$tmp/want.err.n"
    tr -d '\r' <"$tmp/got.err" >"$tmp/got.err.n"
    if cmp -s "$tmp/want.n" "$tmp/got.n" && cmp -s "$tmp/want.err.n" "$tmp/got.err.n"; then
      echo "ok|mir/$rel"
    else
      echo "fail|mir/$rel|want=$(head -c 180 "$tmp/want.n" | tr '\n' ' ')|got=$(head -c 180 "$tmp/got.n" | tr '\n' ' ')|err=$(head -c 180 "$tmp/got.err.n" | tr '\n' ' ')"
      fails=$((fails+1))
    fi
  done
done
echo "done|fails=$fails"
[ "$fails" -eq 0 ]
