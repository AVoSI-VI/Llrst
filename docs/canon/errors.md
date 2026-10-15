# Canon: errors

## Absence

Return `(opt T)`. Never a sentinel like `-1` or empty string.

```
(fn find_byte ((s (& str)) (b u8)) (opt i64) ...)
(match (find_byte (& s) 10u8)
  ((opt.some i) ...)
  ((opt.none) ...))
```

## Failure

Return `(res T str)` with a machine message in pipe form: `"io.read|path=x"`.
The error type in `std` and tools is always `str`. Programs may define an enum instead.

## Propagation

```
(let f (try (fs.read (& p))))       ; yes: try returns early with the err
```
`try` works on `res` and `opt` inside functions returning the same kind.

## Unwrapping

```
(opt.unwrap x)                      ; only in tests and main
(res.unwrap x)
(opt.or x default)                  ; when a default exists
```

## Panics

`(panic "code|k=v")` prints `P|FILE|LINE|COL|code|k=v` and exits 101. Only for invariants
that cannot be violated by input. Input problems are `res.err`.

## Exit codes

| code | meaning |
|------|---------|
| 0 | ok |
| 1 | tool error (bad args, io) |
| 2 | result negative (no matches, test failed) |
| 101 | panic |
