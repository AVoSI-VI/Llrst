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
  (io.puts (vec.get (& xs) i)))           ; yes: [lo hi), step +1
(for i 3 -1 -1
  (io.puts (str.from_int i)))             ; yes: 3 2 1 0; end stays exclusive
```
`for` is `[lo hi)` over an integer. The index name is `i`, then `j`, then `k`.
The step is an atom after `hi` when a body form follows: a literal or a name, evaluated
once. A positive step tests `<`. A negative step tests `>`. A zero step runs nothing.
The default, with the atom omitted, stays `[lo hi)`.

## Conditional loops

```
(while (< i n) ...)                       ; yes when the condition is known up front
(loop ... (if done (brk)) ...)            ; yes when exit is mid-body
(loop outer
  (loop (if done (brk outer))))           ; yes: the name targets that loop
```
`(while ...)` is unchanged. `(brk)` and `(cont)` with no name target the innermost
`loop`, `while`, or `for`. In a `for`, `(cont)` goes to the step. `(brk NAME)` and
`(cont NAME)` target `(loop NAME ...)`.

## Matching

```
(match r
  ((res.ok v) (use v))
  ((res.err e) (io.eputs (& e))))         ; yes: exhaustive, variants in declaration order
(match n
  (0 (if (> n 0)) 1)                      ; guard: (PAT (if EXPR) EXPR); a failing guard tries the next arm
  ((or 1 2) 3)                            ; or-pattern; every alternative binds the same names
  (_ 0))
(match s (("hi" 1) (_ 0)))                ; string literal, compared with str.eq
(match p ((point x y) x))                 ; struct pattern, fields in declaration order
```
A guard does not count as covering its pattern. `match.missing` still reports what is left.
Wildcard `_` only when 3+ variants are ignored.

## Early return

`(ret x)` only in guard clauses at the top of a function. Body end returns by value.

## Booleans

`(and a b)` `(or a b)` `(not a)`. Both short-circuit.
