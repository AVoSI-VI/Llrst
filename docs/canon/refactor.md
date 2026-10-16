# Canon: refactoring

Order of work for any change: reuse, refactor, then new code.

## Reuse

1. `llr-grep -e "fn NAME" std/` before writing a helper.
2. If a std function does 80% of the job, add a parameter or a sibling function in std.

## Extract a function

Trigger: a function passes 30 lines, or the same 3+ forms appear twice.

```
;; before
(fn main () unit
  (let s (str.cat (& a) "|"))
  (let s2 (str.cat (& s) (& b)))
  (io.puts (& s2)))
;; after
(fn pipe2 ((a (& str)) (b (& str))) str
  (let s (str.cat a "|"))
  (str.cat (& s) b))
```
Because every form is a parenthesized tree, extraction is: cut the forms, wrap in
`(fn NAME ARGS RET ...)`, replace with a call. `llr-fmt` renormalizes layout.

## Move to a module

Trigger: a file passes 400 lines. Create `name.llr`, move a cohesive set of `fn`s, mark the
needed ones `pub`, add `(use proj.name)` at call sites, run `llr-test`.

## Rename

```
llr-grep --word OLD DIR        ; list every use
llr-edit --all FILE OLD NEW    ; per file, exact word replace
llr-test DIR
```

## Signature change

Change the `fn`, compile, follow `E|...|type.arity` and `type.mismatch` lines one at a time.
Each line carries file, line, col, and `fix=` when the compiler can compute it.
