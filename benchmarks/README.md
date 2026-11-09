# Benchmarks

Wall-clock comparison of Llrst with Python, Rust, Nim, C, JavaScript, Go, Zig, and Java, plus a regression gate for Llrst itself.

```
benchmarks/run.sh [--check] [--update-baseline] [--once] [--only NAME]
```

`benchmarks/run.ps1` is the same runner on Windows. `NAME` is a task directory, `micro`, `compile`, or `cross`. `--once` takes one sample instead of five.

## What is measured

Each task prints one checksum line. `run.sh` diffs it against `expected.txt` before it keeps a time. Cross-language times are the median of five process runs, in milliseconds, after a separate compile. Llrst micro benches are `llr-bench` on `micro/*.llr` and are nanoseconds per call. Compile lines are one wall-clock sample: the three bootstrap stages of `self/llrs.llr`, then `check` of `std/http.llr`.

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

## First baseline

Linux x86-64, Intel Core Ultra 5 125H, 2026-11-08. Median of five runs, milliseconds, from `baseline.txt`. Stage 2 over stage 0 is 1.15x on `fib`, 1.13x on `sieve`, 1.25x on `nbody`, and 2.8x on `mandelbrot`. Stage 0 against C on that run is 21x on `fib`, 25x on `sieve`, 16x on `string-build`, and 51x on `json-parse`. Short C and Rust times move between runs, so a ratio against a 10 ms program is coarser than the Llrst milliseconds. `llrc` builds `self/llrs.llr` in 4.5s; `llrs0` takes 21.3s; `llrs1` takes 26.0s. Ratios are also in `HANDOFF.md`.
