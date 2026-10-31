#!/usr/bin/env bash
# Fails when any source file in the repo reaches 1000 lines; warns at 500.
root="$(cd "$(dirname "$0")/.." && pwd)"
status=0
while IFS= read -r f; do
  n="$(wc -l <"$f")"
  if [ "$n" -ge 1000 ]; then
    echo "fail|lines|file=${f#$root/}|lines=$n|max=999"
    status=1
  elif [ "$n" -ge 500 ]; then
    echo "W|lines|file=${f#$root/}|lines=$n|fix=split under 500"
  fi
done < <(find "$root/compiler/src" "$root/std" "$root/ui" "$root/web" "$root/tools" "$root/docs" "$root/scripts" "$root/tests" "$root/self" "$root/runtime" \
          -type f \( -name '*.rs' -o -name '*.llr' -o -name '*.md' -o -name '*.sh' -o -name '*.c' -o -name '*.m' \) 2>/dev/null)
[ "$status" -eq 0 ] && echo "ok|lines"
exit $status
