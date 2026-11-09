# llr-bench

Times each `(bench NAME BODY...)` form in one file.

```
llr-bench FILE
```

| flag | default | meaning |
|------|---------|---------|
| `FILE` | | a `.llr` file |

`NAME` is an atom. `BODY` is a unit sequence, the same shape as a `(test ...)` body. The file has no `main`. `llr-bench` rewrites the file: each bench becomes `(fn llr_bench_N () unit BODY)`, then `llrc build` runs it. `$LLRC` is the compiler when set (flags included, as for `llr-test`); otherwise `llrc`.

Each bench is called until one round takes at least 20 ms, or 1048576 calls, whichever comes first. The clock is `time.mono_ms`. The printed count is nanoseconds per call: `(elapsed_ms * 1000000) / calls`.

Output grammar:

```
bench|NAME|ns
```

`|` and newline inside `NAME` are escaped by `fmt.field`.

Exit: 0 timed, 2 no `(bench ...)` form, 1 a bad name, a `main`, a read error, or a failed build (`E|<bench>|0|0|0|bench.name`, `bench.main`, or the compiler's own `E|` lines).

Canon file:

```
(fn add ((a i64) (b i64)) i64 (+ a b))
(bench add
  (let n (add 1 2)))
```
