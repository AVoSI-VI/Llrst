# Workflow: debug

Goal: turn a machine diagnostic into a fix.

## Compile errors

1. `llrc check FILE` (or `llrc build`). Each `E|FILE|LINE|COL|LEN|CODE|...` is independent;
   fix in file order, recompile after each.
2. Map CODE to action:

| code | action |
|------|--------|
| `parse.*` | parens or arity; `form=`/`want=` say which form and how many items |
| `name.unknown` | missing `(use std.X)` or typo; `llr-grep "\\(fn NAME"` to find it |
| `name.private` | item is not `pub` in `module=`; wrap its definition in `(pub ...)` or call a pub wrapper |
| `type.mismatch` | `want=`/`got=`; add a cast `(as T e)`, a borrow `(& e)`, or fix the call |
| `type.infer` | add explicit type args `(vec.new <T>)` or a `let` type |
| `type.arity` | argument count; read the signature in `docs/std/` |
| `match.missing` | add the arms listed in `missing=` |
| `move.used` | value moved at `moved_at=L:C`; borrow instead, or `str.clone` |
| `move.field` / `move.deref` | borrow the field `(& (. p x))` or use `mem.replace` |
| `borrow.mut_shared` / `borrow.mut_mut` | `ref=` is still live (it may be a container such as a vec of refs); shorten its use, reassign it, or clone |
| `borrow.assign` | writing a borrowed place; move the write after the last use of `ref=` |
| `borrow.escape` | returning a reference to a local; return the owned value |
| `mut.required` | `fix=(let mut NAME ...)` |
| `flow.brk_outside` | `brk`/`cont` outside a loop |
| `link.failed` | C toolchain missing; see `docs/install.md` |

3. `llrc emit-mir FILE` prints the MIR when a borrow error is unclear: find the `&` that
   creates the loan and the last use of its holder.

## Runtime panics

`P|FILE|LINE|COL|CODE|k=v` on stderr, exit 101. FILE is usually the std frame that caught
the fault (`std/vec.llr` for `vec.get`), not your call site: `llr-grep "vec\\.get" FILE` in
your code to find candidates, then compare `len=`/`idx=` (or `len= lo= hi=` for slices)
with the values you passed. `opt.none` / `res.err` come from `unwrap`: match instead.
`div.zero` is a `P|` line at the `/` or `%` expression. A process killed by a signal
with no `P|` line is a bug in std, or the minimum signed value divided by `-1`.

## Test failures

`llr-test` prints `fail|TEST|code=101` and the child's `F|FILE|LINE|COL|assert_eq|want=|got=`.
Open FILE at LINE; `want` is the second argument of the assert.
