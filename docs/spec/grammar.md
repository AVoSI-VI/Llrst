# Llrst grammar

File extension: `.llr`. Encoding: UTF-8. Audience: models. One form per task.

## Lexical

```
file    := form*
form    := atom | string | '(' form* ')'
atom    := [^()"\s;]+          ; identifiers, numbers, keywords, dotted names, 'a lifetimes
string  := '"' ( [^"\\] | '\\' [nrt0"\\] | '\\x' hex hex )* '"'
comment := ';' .* eol
```

Only three delimiters exist: `(` `)` `"`. Whitespace separates atoms. There is no infix,
no precedence, no commas, no semicolons, no blocks by indentation.

## Top-level forms

```
(mod NAME)                                   ; declares this file is module NAME
(use PATH)                                   ; import module; PATH = std.io | my.mod
(fn NAME ARGS RET BODY...)                   ; function
(fn NAME <T U> ARGS RET BODY...)             ; generic function
(inline (fn ...))                            ; substitute a body of at most 4 expressions
(pub (inline (fn ...)))                      ; public form of the same
(extern NAME ARGS RET)                       ; C ABI import, no body
(struct NAME FIELD...)                       ; (struct point (x i64) (pub (y i64)))
(struct NAME <T> FIELD...)                   ; generic struct
(enum NAME VARIANT...)                       ; (enum shape (circle f64) (rect f64 f64) (none))
(enum NAME <T> VARIANT...)
(impl TYPE FN...)                            ; methods; called as TYPE.NAME
(impl TYPE <T> FN...)
(const NAME TYPE EXPR)                       ; compile-time scalar or string literal
(test NAME BODY...)                          ; test body, run by llr-test
(pub FORM)                                   ; export; FORM is fn extern struct enum const (per method inside impl)

ARGS   := ( (NAME TYPE)* )
FIELD  := (NAME TYPE) | (pub (NAME TYPE))   ; private unless pub
VARIANT:= (NAME TYPE*) | (pub (NAME TYPE*))   ; private unless pub
```

## Types

```
TYPE := i8 | i16 | i32 | i64 | u8 | u16 | u32 | u64 | f32 | f64 | bool | unit | str
      | NAME                                 ; struct or enum
      | (NAME TYPE...)                       ; generic instance: (vec str) (opt i64) (res i64 str)
      | (& TYPE) | (&mut TYPE)               ; references
      | (& 'a TYPE) | (&mut 'a TYPE)         ; references with named lifetime
      | (ptr TYPE)                           ; raw pointer, std internals only
      | (fn (TYPE...) TYPE)                  ; function pointer
      | T                                    ; generic parameter in scope
```

## Expressions

```
EXPR :=
  LIT                                        ; 1  1.5  true  "s"  'a' is NOT a char literal (use 97u8)
  NAME                                       ; local or const
  (let NAME EXPR)                            ; immutable binding, type inferred
  (let NAME TYPE EXPR)                       ; immutable binding, typed
  (let mut NAME EXPR) | (let mut NAME TYPE EXPR)
  (let mut NAME TYPE)                        ; no initializer; a use before assignment is init.uninit
  (set PLACE EXPR)                           ; assignment
  (do EXPR...)                               ; sequence, value of last
  (if EXPR EXPR) | (if EXPR EXPR EXPR)       ; else branch optional (unit)
  (while EXPR BODY...)
  (loop BODY...)                             ; exit with (brk)
  (for NAME EXPR EXPR BODY...)               ; integer range [lo, hi)
  (brk) | (cont) | (ret) | (ret EXPR)
  (match EXPR ARM...)                        ; ARM := (PAT EXPR)
  (& PLACE) | (&mut PLACE) | (@ EXPR)        ; borrow, mutable borrow, deref
  (. EXPR NAME)                              ; field access
  (as TYPE EXPR)                             ; numeric cast
  (OP EXPR EXPR)                             ; + - * / % == != < <= > >= band bor bxor shl shr wadd wsub wmul
  (and EXPR EXPR...) | (or EXPR EXPR...) | (not EXPR)  ; and/or take 2 or more
  (neg EXPR) | (wneg EXPR)                   ; wneg always wraps
  (CALLEE EXPR...)                           ; call: fn, mod.fn, type.method, type (struct ctor), enum.variant
  (CALLEE <TYPE...> EXPR...)                 ; call with explicit type args
  (try EXPR)                                 ; unwrap res/opt or return the err/none from the fn
  (panic EXPR)                               ; print P|... and exit 101
  (assert EXPR) | (assert_eq EXPR EXPR)      ; test asserts
  (sizeof TYPE)                              ; u64, std internals only

PLACE := NAME | (. PLACE NAME) | (@ EXPR)
PAT   := _ | NAME | LIT | (ENUM.VARIANT PAT...) | (ENUM.VARIANT)
```

Literal suffixes select integer type: `7u8`, `7i32`, `7u64`. Unsuffixed defaults to `i64`,
or to the expected type when known. Floats: `1.0` default `f64`, `1.0f32`.

A function body is a sequence; the last expression is the return value unless it is a
statement form (`let`, `set`, `while`, `for`, `loop`), in which case the body returns unit.

Struct construction is positional: `(point 1 2)`. Enum construction: `(shape.circle 1.0)`,
`(opt.none)`. Field read `(. p x)`. Field write `(set (. p x) 3)`.

String literals have type `(& str)` and point to static data. Owned strings come from
`str.from`, `str.cat`, etc.

## Example

```
(use std.io)
(fn main () unit
  (io.puts "hello world"))
```
