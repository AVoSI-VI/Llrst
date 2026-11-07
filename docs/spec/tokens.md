# Tokens

Design goal: minimal distinct tokens, each short, each one BPE token where possible.

## Delimiters (3)

`(` `)` `"`

## Keywords (all lowercase)

| kw | role |
|----|------|
| `fn` | function |
| `let` | binding |
| `mut` | mutable modifier for `let` |
| `set` | assignment |
| `do` | sequence |
| `if` | conditional |
| `while` | loop with condition |
| `loop` | infinite loop |
| `for` | integer range loop |
| `brk` | break |
| `cont` | continue |
| `ret` | return |
| `match` | pattern match |
| `struct` | struct decl |
| `enum` | enum decl |
| `impl` | methods |
| `extern` | C import |
| `const` | constant |
| `use` | import |
| `mod` | module name |
| `pub` | export |
| `test` | test body |
| `as` | cast |
| `and` `or` `not` | booleans |
| `neg` | numeric negate |
| `bnot` | bitwise complement of an integer |
| `wadd` `wsub` `wmul` `wneg` | wrapping arithmetic; never traps |
| `band` `bor` `bxor` `shl` `shr` | bitwise |
| `trait` | trait declaration |
| `clos` | closure type and `(clos ...)` value |
| `call` | call a closure |
| `inline` | substitute the function body at the call |
| `try` | early return of `res.err` or `opt.none` |
| `panic` | abort with a `P\|` record |
| `assert` | check a condition |
| `assert_eq` | check two values |
| `sizeof` | compile-time size of a type |
| `dyn` | trait object |
| `ptr` | raw pointer type |
| `never` | type with no value |

## Sigils

| sigil | role |
|-------|------|
| `&` | shared borrow `(& x)` and type `(& T)` |
| `&mut` | mutable borrow and type |
| `@` | deref `(@ r)` |
| `.` | field access `(. p x)` and namespace `io.puts` |
| `'a` | lifetime; `'static` is the lifetime of literals, consts, and other `'static` references |
| `_` | wildcard pattern |
| `<T>` | type parameter list (atoms starting with `<`, ending with `>`) |
| `;` | comment to end of line |

## Operators (prefix, binary)

`+ - * / % == != < <= > >=`

## Names

`[a-z][a-z0-9_]*` for locals, fns, fields, modules. Types use the same form (no CamelCase; it
costs tokens). Dotted name `a.b` means `b` inside module or type `a`.

## Literals

`123` `123u8` `123i32` `0x7F` `0xFFi32` `0b1010` `0o17` `1_000` `1.5` `1.5f32` `1e9` `1.5e-3` `true` `false` `"text"`
An integer may be written in hexadecimal (`0x`), binary (`0b`), or octal (`0o`). Digits above 9 are `a-f` and `A-F`. The same type suffixes as decimal apply (`0x7Fi32`, `0b1010u8`). Prefixes are lowercase. A lowercase `f`, `i`, or `u` starts a suffix, so a hex digit `f` is written `F`. `_` may separate digits (`1_000`, `0xFF_FF`).
A float may use an exponent: `1e9`, `1.5e-3`, `1E+2`. The exponent makes the literal a float even when there is no dot.
String escapes: `\n \r \t \0 \" \\ \xHH \u{HHHH}`. `\u{HHHH}` is one to six hex digits, at most U+10FFFF, not a surrogate, and is encoded as UTF-8.

## Token budget rules (for std and tools authors)

1. Prefer one dotted call over a chain: `(str.cat a b)` not `(. (str.cat a) b)`.
2. No type annotation when the init expression fixes the type.
3. Names 1-5 chars for locals, 2-8 for fns.
4. Do not comment what the code already says.
