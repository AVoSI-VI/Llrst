# Canon: program structure

## Minimal program

```
(use std.io)
(fn main () unit
  (io.puts "hello world"))
```

`main` takes no args and returns `unit`. Exit code via `(os.exit 2)`. Args via `(os.args)`.

## Imports

```
(use std.io)          ; yes: one module per use
(use std.io std.str)  ; no: canon.program.use_one
```
Imports go first in the file, sorted by path. `std.*` modules are referenced as their last
segment: `io.puts`, `str.cat`. `(use ui.NAME)` loads `ui/NAME.llr` the same way, one module
per use. `ui` types are not prelude. The directory is `--ui`, then `$LLRST_UI`, then
`<exe>/../share/llrst/ui`, `<exe>/../../ui`, then `./ui`.

## Modules

One file is one module. The module name is the file name without `.llr`.
`(mod NAME)` is written only when the name differs from the file name (never, in canon).

Every `fn`, `extern`, `struct`, `enum`, `const`, and `impl` method is private to its
file unless wrapped: `(pub (fn ...))`, `(pub (struct ...))`, and inside `(impl T ...)`
each exported method is `(pub (fn ...))`. There is no `(pub (impl ...))`. Using a private
item from another file is `name.private|name=|module=|fix=(pub ...)`. A struct field is
private unless it is `(pub (name type))`. Code in the struct's file, including its `impl`,
may use private fields. Another file gets `name.private_field` on `(. p name)` and on a
positional constructor. `(mem.replace (&mut place) new)` swaps the whole struct and copies
private fields. Another file may do that without naming the fields. `name.private_field`
does not apply. This is the escape. An enum variant is private unless it is `(pub (name type*))`.
Code in the enum's file may construct and match a private variant. Another file gets
`name.private_variant`. A std type enters the prelude only when
it is `pub`.

Import a sibling module with `(use proj.name)` where the root is the directory of the file
holding `main`.

## File order

1. `use`
2. `const`
3. `struct` / `enum`
4. `impl`
5. `fn` (helpers before callers)
6. `fn main` last
7. `test` forms

## Entry point for tools

Tools parse args with `cli`:

```
(use std.cli)
(use std.io)
(use std.os)
(fn main () unit
  (let mut c (cli.new "llr-read" "print lines of a file"))
  (cli.pos (&mut c) "path")
  (cli.opt (&mut c) "from" "1")
  (cli.flag (&mut c) "all")
  (cli.parse (&mut c) (os.args))
  (let path (cli.at (& c) 0))
  (let from (cli.get_int (& c) "from"))
  (if (cli.has (& c) "all") ...))
```
