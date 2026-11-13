# Benchmarks

Wall-clock comparison of Llrst with Python, Rust, Nim, C, JavaScript, Go, Zig, and Java, plus a regression gate for Llrst itself.

```
benchmarks/run.sh [--check] [--update-baseline] [--once] [--only NAME]
```

`benchmarks/run.ps1` is the same runner on Windows. `NAME` is a task directory, `micro`, `compile`, or `cross`. `--once` takes one sample instead of five.

## What is measured

Each task prints one checksum line. `run.sh` diffs it against `expected.txt` before it keeps a time. Cross-language times are the median of five process runs, in milliseconds, after a separate compile. Llrst micro benches are `llr-bench` on `micro/*.llr` and are nanoseconds per call. `sha1` and `sha256` hash 64 KiB. `show_u64` prints a `u64`. Compile lines are one wall-clock sample: the three bootstrap stages of `self/llrs.llr`, then `check` of `std/http.llr`.

A missing compiler prints `skip|<task>|<lang>|not-installed` and is not a failure. If `zig` is on `PATH` but has no `lib` directory beside it, set `ZIG` to a binary that does. The log also prints `spread|<task>|<lang>|<min ms>`; the gated number is the median.

## Fairness

Same size, same algorithm, same checksum. Build lines:

| language | command |
|----------|---------|
| Llrst stage 0 | `llrc build` |
| Llrst stage 2 | `llrs2 build` |
| C | `gcc -O2 -std=c11` |
| Rust | `rustc -O --edition 2021` |
| Nim | `nim c -d:release` |
| Python | `python3` |
| JavaScript | `node` |
| Go | `go build` |
| Zig | `zig build-exe -O ReleaseFast` |
| Java | `javac` then `java` |

No extra dependencies, no SIMD, no threads. Each language uses its usual library where it has one (Python `json`, Go `encoding/json`, Nim `std/json`, Node `JSON.parse`, Llrst `json_read`). C, Rust, Java, and Zig parse the same document with a direct scanner because those standard libraries are not in the box. N-body uses one eight-step Newton square root in every language so the checksum is the integration. Llrst's `math.sqrt` is a different iteration, and libm is a different one again.

## Regression gate

`baseline.txt` is checked in. Its first line is `# machine|<os>|<arch>|<cpu>`. `run.sh --check` fails when a Llrst line (`bench|stage0`, `bench|stage2`, `cross|...|llr-stage0`, `cross|...|llr-stage2`, or any `compile|` line) is more than 30% slower than that file. Two repeats of an unchanged tree moved lines by up to 28%, so 10% fails a clean run. Other languages are reported and never gated. On a different machine the comparison is printed and the exit status stays 0.

`run.sh --update-baseline` rewrites `baseline.txt` from the run just finished. Do that in the same commit as the change that moved a number, and say which lines moved.

Stage 0 is `target/release/llrc`. `LLRC` is not read, because that variable is often the installed stage 2 compiler. `LLRC0` overrides stage 0. `LLRS2` overrides stage 2.

## After a perf change

On the machine named in `baseline.txt`:

```
benchmarks/run.sh --check
```

That is the gate for a commit that touches `self/cg_*.llr`, `self/isa_*.llr`, `self/inline.llr`, `compiler/src/codegen/`, `compiler/src/mir/`, or a hot `std/` file. It exits 1 when a Llrst line (`bench|stage0`, `bench|stage2`, `cross|...|llr-stage0`, `cross|...|llr-stage2`, or `compile|`) is more than 30% slower than `baseline.txt`. Other languages are printed and never fail the run. On a different machine the same comparison is printed and the exit status stays 0.

When the change is meant to move a number, run `benchmarks/run.sh --update-baseline` in that same commit and name the lines that moved. `--check` alone does not rewrite the file.

## Current baseline

Linux x86-64, Intel Core Ultra 5 125H, 2026-11-09, median of five, in `baseline.txt`. Stage 2 beats Python on every task except `json_parse`, and beats Node on `fib`, `sort`, `string_build`, and `map_ops`. The other Node gaps, and every Rust, C, and Nim gap over 2x, are one line each in `langspeed.md` Phase 5. `llrc` builds `self/llrs.llr` in 3.0s; `llrs0` takes 4.2s; `llrs1` takes 5.9s.

The first baseline (2026-11-08) had stage 2 at 1.15x stage 0 on `fib` and 2.8x on `mandelbrot`, and stage 0 at 21x C on `fib` and 51x C on `json-parse`. Those rows are the history, not the gate.
