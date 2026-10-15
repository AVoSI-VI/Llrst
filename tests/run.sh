#!/usr/bin/env bash
# Golden tests: tests/run/*.llr (stdout must equal .out), tests/err/*.llr (stderr must
# equal .err), and every std/*.llr compiled with --test. Output: ok|NAME or fail|NAME|...
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
llrc="${LLRC:-$root/target/release/llrc}"
[ -x "$llrc" ] || llrc="$(command -v llrc || true)"
if [ -z "$llrc" ] || [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release or set LLRC"
  exit 1
fi
export LLRST_STD="$root/std"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
fails=0

for f in "$root"/tests/run/*.llr; do
  name="$(basename "$f" .llr)"
  want="${f%.llr}.out"
  if ! "$llrc" build "$f" -o "$tmp/$name" 2>"$tmp/$name.err"; then
    echo "fail|run/$name|compile|$(head -1 "$tmp/$name.err")"; fails=$((fails+1)); continue
  fi
  "$tmp/$name" >"$tmp/$name.out" 2>/dev/null
  if cmp -s "$tmp/$name.out" "$want"; then
    echo "ok|run/$name"
  else
    echo "fail|run/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.out" | tr '\n' ' ')"
    fails=$((fails+1))
  fi
done

for f in "$root"/tests/err/*.llr; do
  [ -e "$f" ] || continue
  name="$(basename "$f" .llr)"
  want="${f%.llr}.err"
  ( cd "$root" && "$llrc" check "tests/err/$name.llr" 2>"$tmp/$name.err" >/dev/null )
  if cmp -s "$tmp/$name.err" "$want"; then
    echo "ok|err/$name"
  else
    echo "fail|err/$name|want=$(head -c 200 "$want" | tr '\n' ' ')|got=$(head -c 200 "$tmp/$name.err" | tr '\n' ' ')"
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
