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
| `name.private_field` | field is private in `module=`; write `(pub (field type))` or call a method in that module |
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
| `link.failed` | C toolchain missing or the link command failed; `cmd=` `got=`; see `docs/install.md` |
| `name.dup` | the same name is declared twice in one module; rename one |
| `name.not_module` | `(use ...)` does not name a loaded module; `name=` |
| `name.not_type` | a type name is not defined; `name=` |
| `name.private_variant` | variant is private; `fix=(pub (name types))` in `module=` |
| `type.not_callable` | the callee is not a function; `got=` |
| `type.no_field` | `ty=` has no `field=` |
| `type.no_variant` | `ty=` has no `variant=` |
| `type.not_numeric` | the operator needs a number; `got=` |
| `type.not_bool` | `if` / `while` / `and` / `or` need `bool`; `got=` |
| `type.generic_arity` | wrong number of type arguments; `want=` `got=` |
| `init.uninit` | `name=` is read on a path that never assigns it |
| `move.borrowed` | the place is still borrowed at `borrowed_at=`; end the borrow first |
| `inline.reject` | `why=extern` cannot be inlined; `why=form\|got=clos` means the body contains a closure |
| `clos.nested` | move the inner closure out; a closure body cannot contain a closure |
| `trait.missing` | add `(impl TRAIT TYPE ...)`; `trait=` `ty=` |
| `trait.dup` | two impls of the same trait for `ty=` |
| `trait.object` | `why=self` means the method returns or takes `Self`; `why=ref` means a trait object was written as a bare name |
| `trait.method` | the impl is missing `name=` or its signature does not match |
| `io.read` | `path=` could not be read; `fix=set LLRST_STD` or add the `proj.` file named by `use=` |
| `lex.*` | close the string or fix the escape; `lex.bad_suffix` means the literal suffix is not a type |
| `target.unsupported` | `--target` is not an `x86_64` triple this compiler emits; `got=` |
| `use.unused` | delete the `(use ...)`; `name=` is the alias. `std` modules are not linted |
| `fn.unused` | delete the private function or call it. `pub`, `extern`, `main`, and `std` are not linted |

3. `llrc emit-mir FILE` prints the MIR when a borrow error is unclear: find the `&` that
   creates the loan and the last use of its holder.

A `W|` line is not a failure. `name.unused` means delete the binding or use it.
`flow.unreachable` means the statements after a `(ret)` or an always-taken branch are dead.

## Runtime panics

`P|FILE|LINE|COL|CODE|k=v` on stderr, exit 101, then one `P|name` line per live caller,
innermost first. `sys.abort` and `sys.panic` are omitted. An inlined body has its own
frame (`vec.check`, `str.at`). Drop glue adds `drop.<type>`. FILE is often the std file that detected the fault (`std/vec.llr` for an out-of-line
`vec.insert`), so the following `P|name` lines name the callers. `opt.none` / `res.err` come
from `unwrap`: match instead.
`div.zero` is a `P|` line at the `/` or `%` expression. Signed `MIN / -1` and `MIN % -1`
always panic `int.overflow` at that expression. Other `int.overflow` lines mean the program
was built with `--checked` and a `+` `-` `*` `neg` or an out-of-range shift overflowed; use
`wadd` `wsub` `wmul` `wneg` when the wrap is intended. A process killed by a signal
with no `P|` line is a bug in std.

## Test failures

`llr-test` prints `fail|TEST|code=101` and the child's `F|FILE|LINE|COL|assert_eq|want=|got=`.
Open FILE at LINE; `want` is the second argument of the assert.
