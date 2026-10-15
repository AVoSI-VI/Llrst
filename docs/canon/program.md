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
segment: `io.puts`, `str.cat`.

## Modules

One file is one module. The module name is the file name without `.llr`.
`(mod NAME)` is written only when the name differs from the file name (never, in canon).
Export with `(pub (fn ...))`. Everything else is private to the file.

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
  (let a (cli.parse (&mut c) (os.args)))
  ...)
```
