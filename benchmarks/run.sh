#!/usr/bin/env bash
# Time Llrst against the other languages and the two compilers against each other.
#   benchmarks/run.sh [--check] [--update-baseline] [--once] [--only NAME]
# --once is one sample instead of five. --only limits the run to one task name,
# "micro", or "compile". Checksum mismatches always fail. --check fails when a
# Llrst line is more than 30% slower than baseline.txt on the same machine.
set -u
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root" || exit 1

once=0
check=0
update=0
only=""
while [ $# -gt 0 ]; do
  case "$1" in
    --once) once=1 ;;
    --check) check=1 ;;
    --update-baseline) update=1 ;;
    --only)
      shift
      only="${1:-}"
      if [ -z "$only" ]; then
        echo "fail|args|--only"
        exit 1
      fi
      ;;
    *)
      echo "fail|args|$1"
      exit 1
      ;;
  esac
  shift
done

runs=5
if [ "$once" -eq 1 ]; then
  runs=1
fi

results="$root/benchmarks/results"
mkdir -p "$results/compile"
log="$results/last.txt"
: >"$log"

# LLRC is the installed compiler in this repo and is often llrs2. Stage 0 is the
# Rust binary. Set LLRC0 only to point stage 0 somewhere else.
llrc="${LLRC0:-$root/target/release/llrc}"
llrs2="${LLRS2:-$root/target/self/llrs2}"
if [ ! -x "$llrc" ]; then
  echo "fail|setup|llrc=not found|fix=cargo build --release"
  exit 1
fi
if [ ! -x "$llrs2" ]; then
  echo "fail|setup|llrs2=not found|fix=scripts/self.sh --stage"
  exit 1
fi
if [ "$(readlink -f "$llrc")" = "$(readlink -f "$llrs2")" ]; then
  echo "fail|setup|stage0=stage2|llrc=$llrc"
  exit 1
fi

export LLRST_STD="$root/std"
export LLRST_RUNTIME="$root/runtime"

zig_works() {
  local bin="$1" dir
  [ -n "$bin" ] && [ -x "$bin" ] || return 1
  dir="$(cd "$(dirname "$bin")" && pwd)"
  [ -d "$dir/lib" ] || return 1
  "$bin" version >/dev/null 2>&1
}

zigbin=""
if zig_works "${ZIG:-}"; then
  zigbin="$ZIG"
elif command -v zig >/dev/null 2>&1 && zig_works "$(command -v zig)"; then
  zigbin="$(command -v zig)"
else
  for cand in "$HOME"/Downloads/zig-*/zig; do
    if zig_works "$cand"; then
      zigbin="$cand"
      break
    fi
  done
fi
cc -c -fPIC "$root/runtime/llr_stat.c" -o "$root/runtime/llr_stat.o" || exit 1
cc -c -fPIC "$root/runtime/llr_now.c" -o "$root/runtime/llr_now.o" || exit 1
cc -c -fPIC "$root/runtime/llr_reload.c" -o "$root/runtime/llr_reload.o" || exit 1
cc -c -fPIC "$root/runtime/llr_dl.c" -o "$root/runtime/llr_dl.o" || exit 1

model="$(sed -n 's/^model name[[:space:]]*: //p' /proc/cpuinfo 2>/dev/null | head -1)"
if [ -z "$model" ]; then
  model="$(uname -s)"
fi
machine="$(uname -s)|$(uname -m)|$model"

note() { printf '%s\n' "$1" | tee -a "$log"; }

tasks="fib sieve sort string_build map_ops json_parse nbody mandelbrot spectral_norm fannkuch binary_trees"
micros="fib sieve sort string_build map_ops alloc_churn sha1 sha256 show_u64"

one_task=""
do_cross=0
do_micro=0
do_compile=0
if [ -z "$only" ]; then
  do_cross=1
  do_micro=1
  do_compile=1
elif [ "$only" = "micro" ]; then
  do_micro=1
elif [ "$only" = "compile" ]; then
  do_compile=1
elif [ "$only" = "cross" ]; then
  do_cross=1
else
  known=0
  for task in $tasks; do
    if [ "$only" = "$task" ]; then
      known=1
    fi
  done
  if [ "$known" -ne 1 ]; then
    echo "fail|args|$only"
    exit 1
  fi
  do_cross=1
  one_task="$only"
fi

median() {
  python3 -c 'import sys; xs=sorted(int(x) for x in sys.argv[1:]); print(xs[len(xs)//2], xs[0])' "$@"
}

fail() {
  note "fail|$1"
  exit 1
}

time_ms() {
  local start end
  start="$(date +%s%N)"
  "$@"
  local code=$?
  end="$(date +%s%N)"
  TIMED_MS=$(( (end - start) / 1000000 ))
  return "$code"
}

run_samples() {
  local out="$1"
  shift
  local i=0
  local samples=""
  while [ "$i" -lt "$runs" ]; do
    if ! time_ms "$@" >"$out" 2>"$out.err"; then
      return 1
    fi
    samples="$samples $TIMED_MS"
    i=$((i + 1))
  done
  median $samples
}

build_llr() {
  local comp="$1" src="$2" out="$3"
  "$comp" build "$src" -o "$out" --std "$root/std" >"$out.build" 2>&1
}

run_cross() {
  local task="$1"
  local dir="$root/benchmarks/tasks/$task"
  local expect="$dir/expected.txt"
  local slot="$results/$task"
  mkdir -p "$slot"
  local lang bin src
  for lang in llr-stage0 llr-stage2 c rust nim python node go zig java; do
    src=""
    bin="$slot/$lang"
    case "$lang" in
      llr-stage0) src="$dir/$task.llr" ;;
      llr-stage2) src="$dir/$task.llr" ;;
      c) src="$dir/$task.c" ;;
      rust) src="$dir/$task.rs" ;;
      nim) src="$dir/$task.nim" ;;
      python) src="$dir/$task.py" ;;
      node) src="$dir/$task.js" ;;
      go) src="$dir/$task.go" ;;
      zig) src="$dir/$task.zig" ;;
      java) src="$dir/Main.java" ;;
    esac
    if [ ! -f "$src" ]; then
      note "skip|$task|$lang|missing"
      continue
    fi
    case "$lang" in
      llr-stage0)
        build_llr "$llrc" "$src" "$bin" || fail "$task|$lang|build"
        ;;
      llr-stage2)
        build_llr "$llrs2" "$src" "$bin" || fail "$task|$lang|build"
        ;;
      c)
        command -v gcc >/dev/null || { note "skip|$task|c|not-installed"; continue; }
        gcc -O2 -std=c11 -o "$bin" "$src" -lm >"$bin.build" 2>&1 || fail "$task|c|build"
        ;;
      rust)
        command -v rustc >/dev/null || { note "skip|$task|rust|not-installed"; continue; }
        rustc -O --edition 2021 -o "$bin" "$src" >"$bin.build" 2>&1 || fail "$task|rust|build"
        ;;
      nim)
        command -v nim >/dev/null || { note "skip|$task|nim|not-installed"; continue; }
        nim c -d:release --hints:off --warnings:off --nimcache:"$slot/nimcache" -o:"$bin" "$src" >"$bin.build" 2>&1 || fail "$task|nim|build"
        ;;
      python)
        command -v python3 >/dev/null || { note "skip|$task|python|not-installed"; continue; }
        ;;
      node)
        command -v node >/dev/null || { note "skip|$task|node|not-installed"; continue; }
        ;;
      go)
        command -v go >/dev/null || { note "skip|$task|go|not-installed"; continue; }
        (cd "$dir" && go build -o "$bin" "$task.go") >"$bin.build" 2>&1 || fail "$task|go|build"
        ;;
      zig)
        if [ -z "$zigbin" ]; then
          note "skip|$task|zig|not-installed"
          continue
        fi
        mkdir -p "$results/zig-cache" "$results/zig-global"
        "$zigbin" build-exe -O ReleaseFast -femit-bin="$bin" \
          --cache-dir "$results/zig-cache" --global-cache-dir "$results/zig-global" \
          "$src" >"$bin.build" 2>&1 || fail "$task|zig|build"
        ;;
      java)
        # macOS has a /usr/bin/javac stub that only prints an install prompt.
        javac -version >/dev/null 2>&1 || { note "skip|$task|java|not-installed"; continue; }
        mkdir -p "$slot/java"
        javac -d "$slot/java" "$src" >"$bin.build" 2>&1 || fail "$task|java|build"
        ;;
    esac
    local pair med mn
    case "$lang" in
      python) pair="$(run_samples "$slot/$lang.out" python3 "$src")" || fail "$task|python|run" ;;
      node) pair="$(run_samples "$slot/$lang.out" node "$src")" || fail "$task|node|run" ;;
      java) pair="$(run_samples "$slot/$lang.out" java -cp "$slot/java" Main)" || fail "$task|java|run" ;;
      *) pair="$(run_samples "$slot/$lang.out" "$bin")" || fail "$task|$lang|run" ;;
    esac
    med="${pair%% *}"
    mn="${pair##* }"
    if ! cmp -s "$slot/$lang.out" "$expect"; then
      note "fail|$task|$lang|stdout"
      echo "got $(cat "$slot/$lang.out")"
      echo "want $(cat "$expect")"
      exit 1
    fi
    note "cross|$task|$lang|$med"
    note "spread|$task|$lang|$mn"
  done
}

run_micro() {
  local stage="$1" comp="$2" name
  for name in $micros; do
    local i=0 samples="" ns out pair
    while [ "$i" -lt "$runs" ]; do
      if ! out="$(LLRC="$comp" "$llrc" run "$root/tools/llr-bench.llr" -- "$root/benchmarks/micro/$name.llr" 2>"$results/micro-$stage-$name.err")"; then
        fail "micro|$stage|$name"
      fi
      ns="$(printf '%s\n' "$out" | awk -F'|' '/^bench\|/ { print $3; exit }')"
      if [ -z "$ns" ]; then
        fail "micro|$stage|$name|parse"
      fi
      samples="$samples $ns"
      i=$((i + 1))
    done
    pair="$(median $samples)"
    note "bench|$stage|$name|${pair%% *}"
  done
}

run_compile() {
  mkdir -p "$results/compile"
  local src="$root/self/llrs.llr"
  export SOURCE_DATE_EPOCH=0
  local start end ms
  start="$(date +%s%N)"
  "$llrc" build "$src" -o "$results/compile/llrs0" --std "$root/std" >"$results/compile/llrs0.log" 2>&1 || fail "compile|llrc|self/llrs.llr"
  end="$(date +%s%N)"
  ms=$(( (end - start) / 1000000 ))
  note "compile|llrc|self/llrs.llr|$ms"
  start="$(date +%s%N)"
  "$results/compile/llrs0" build "$src" -o "$results/compile/llrs1" --std "$root/std" >"$results/compile/llrs1.log" 2>&1 || fail "compile|llrs0|self/llrs.llr"
  end="$(date +%s%N)"
  ms=$(( (end - start) / 1000000 ))
  note "compile|llrs0|self/llrs.llr|$ms"
  start="$(date +%s%N)"
  "$results/compile/llrs1" build "$src" -o "$results/compile/llrs2" --std "$root/std" >"$results/compile/llrs2.log" 2>&1 || fail "compile|llrs1|self/llrs.llr"
  end="$(date +%s%N)"
  ms=$(( (end - start) / 1000000 ))
  note "compile|llrs1|self/llrs.llr|$ms"
  start="$(date +%s%N)"
  "$llrc" check "$root/std/http.llr" --std "$root/std" >"$results/compile/http-llrc.log" 2>&1 || fail "compile|llrc|std/http.llr"
  end="$(date +%s%N)"
  ms=$(( (end - start) / 1000000 ))
  note "compile|llrc|std/http.llr|$ms"
  start="$(date +%s%N)"
  "$llrs2" check "$root/std/http.llr" --std "$root/std" >"$results/compile/http-llrs2.log" 2>&1 || fail "compile|llrs2|std/http.llr"
  end="$(date +%s%N)"
  ms=$(( (end - start) / 1000000 ))
  note "compile|llrs2|std/http.llr|$ms"
  unset SOURCE_DATE_EPOCH
}

if [ "$do_cross" -eq 1 ]; then
  for task in $tasks; do
    if [ -z "$one_task" ] || [ "$one_task" = "$task" ]; then
      run_cross "$task"
    fi
  done
fi
if [ "$do_micro" -eq 1 ]; then
  run_micro stage0 "$llrc"
  run_micro stage2 "$llrs2"
fi
if [ "$do_compile" -eq 1 ]; then
  run_compile
fi

if [ "$update" -eq 1 ]; then
  {
    echo "# machine|$machine"
    echo "# clock|time.mono_ns"
    grep -E '^(bench|cross|compile)\|' "$log"
  } >"$root/benchmarks/baseline.txt"
  note "ok|baseline|benchmarks/baseline.txt"
fi

if [ "$check" -eq 1 ]; then
  base="$root/benchmarks/baseline.txt"
  [ -f "$base" ] || fail "check|baseline=missing"
  base_m="$(sed -n 's/^# machine|//p' "$base" | head -1)"
  advisory=0
  if [ "$base_m" != "$machine" ]; then
    note "note|machine|baseline=$base_m|now=$machine|advisory"
    advisory=1
  fi
  status=0
  while IFS= read -r line; do
    case "$line" in
      bench\|*|compile\|*) ;;
      cross\|*)
        lang="$(printf '%s\n' "$line" | awk -F'|' '{print $3}')"
        case "$lang" in
          llr-stage0|llr-stage2) ;;
          *) continue ;;
        esac
        ;;
      *) continue ;;
    esac
    key="$(printf '%s\n' "$line" | awk -F'|' '{ OFS="|"; NF--; print }')"
    now="$(printf '%s\n' "$line" | awk -F'|' '{print $NF}')"
    prev="$(awk -F'|' -v k="$key" 'index($0, k "|")==1 { v=$NF } END { print v }' "$base")"
    if [ -z "$prev" ]; then
      note "fail|check|$key|missing"
      status=1
      continue
    fi
    limit=$(( prev + prev * 3 / 10 ))
    if [ "$now" -gt "$limit" ]; then
      pct=$(( (now - prev) * 100 / prev ))
      note "fail|$key|base=$prev|now=$now|+$pct%"
      status=1
    fi
  done < <(grep -E '^(bench|cross|compile)\|' "$log")
  if [ "$status" -ne 0 ] && [ "$advisory" -eq 1 ]; then
    note "note|check|advisory"
    status=0
  fi
  if [ "$status" -ne 0 ]; then
    exit 1
  fi
  note "ok|check"
fi

note "ok|bench"
exit 0
