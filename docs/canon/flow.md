# Canon: control flow

## Conditionals

```
(if (< a b) a b)                          ; yes: expression form
(if ok (do (io.puts "a") (io.puts "b")))  ; yes: multi-statement branch uses do
(if (not ok) (ret))                       ; yes: guard clause
```
No `else if` chain; use `match` on a value or nested `if` at most 2 deep. Deeper: extract fn.

## Counting loops

```
(for i 0 (vec.len (& xs))
  (io.puts (vec.get (& xs) i)))           ; yes
```
`for` is always `[lo hi)` over `i64`. The index name is `i`, then `j`, then `k`.

## Conditional loops

```
(while (< i n) ...)                       ; yes when the condition is known up front
(loop ... (if done (brk)) ...)            ; yes when exit is mid-body
```

## Matching

```
(match r
  ((res.ok v) (use v))
  ((res.err e) (io.eputs (& e))))         ; yes: exhaustive, variants in declaration order
```
Wildcard `_` only when 3+ variants are ignored.

## Early return

`(ret x)` only in guard clauses at the top of a function. Body end returns by value.

## Booleans

`(and a b)` `(or a b)` `(not a)`. Both short-circuit.
