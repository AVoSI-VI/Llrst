# Tokens

Design goal: minimal distinct tokens, each short, each one BPE token where possible.

## Delimiters (3)

`(` `)` `"`

## Keywords (all lowercase, 2-6 chars)

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
| `band` `bor` `bxor` `shl` `shr` | bitwise |

## Sigils

| sigil | role |
|-------|------|
| `&` | shared borrow `(& x)` and type `(& T)` |
| `&mut` | mutable borrow and type |
| `@` | deref `(@ r)` |
| `.` | field access `(. p x)` and namespace `io.puts` |
| `'a` | lifetime |
| `_` | wildcard pattern |
| `<T>` | type parameter list (atoms starting with `<`, ending with `>`) |
| `;` | comment to end of line |

## Operators (prefix, binary)

`+ - * / % == != < <= > >=`

## Names

`[a-z][a-z0-9_]*` for locals, fns, fields, modules. Types use the same form (no CamelCase; it
costs tokens). Dotted name `a.b` means `b` inside module or type `a`.

## Literals

`123` `123u8` `123i32` `1.5` `1.5f32` `true` `false` `"text"`
String escapes: `\n \r \t \0 \" \\ \xHH`.

## Token budget rules (for std and tools authors)

1. Prefer one dotted call over a chain: `(str.cat a b)` not `(. (str.cat a) b)`.
2. No type annotation when the init expression fixes the type.
3. Names 1-5 chars for locals, 2-8 for fns.
4. Do not comment what the code already says.
