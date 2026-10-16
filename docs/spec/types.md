# Types

## Primitives

| type | size | notes |
|------|------|-------|
| `i8 i16 i32 i64` | 1 2 4 8 | signed |
| `u8 u16 u32 u64` | 1 2 4 8 | unsigned |
| `f32 f64` | 4 8 | IEEE |
| `bool` | 1 | `true` `false` |
| `unit` | 0 | the only value is `()`; functions with nothing to return use it |
| `str` | 24 | owned heap string `{ptr len cap}`; literal `"x"` is `(& str)` |

## References

`(& T)` shared, `(&mut T)` unique. Pointer-sized. Governed by the borrow rules in
[memory.md](memory.md). `(& 'a T)` names the lifetime `'a`, needed only when a function returns
a reference and takes more than one reference parameter.

## Raw pointers

`(ptr T)`: pointer-sized, no safety. Only `std` uses them. Ops: `(ptr.null T)`,
`(ptr.add p n)`, `(ptr.read p)`, `(ptr.write p v)`, `(ptr.eq a b)`, `(as (ptr u8) p)`.

## Structs

```
(struct point (x i64) (y i64))
(let p (point 1 2))
(. p x)
```
Layout: fields in declaration order, natural alignment, size rounded up to max alignment.
Passed by pointer at the ABI level; semantically moved by value.

## Enums

```
(enum shape (circle f64) (rect f64 f64) (none))
(shape.circle 1.0)
(match s ((shape.circle r) r) ((shape.rect w h) (* w h)) (_ 0.0))
```
Layout: `u32` tag then payload union, aligned to max payload alignment.

## Generics

```
(fn first <T> ((v (& (vec T)))) (& T) (vec.get v 0))
(struct pair <A B> (a A) (b B))
(impl pair <A B> (fn swap ((self pair)) (pair B A) (pair (. self b) (. self a))))
```
Generics are monomorphized: each distinct instantiation is compiled separately. Type
arguments are inferred from argument types and from the expected type; otherwise pass them
explicitly: `(vec.new <str>)`.

## Standard generic types (std)

| type | role |
|------|------|
| `(vec T)` | growable array `{ptr len cap}` |
| `(opt T)` | `(opt.some x)` or `(opt.none)` |
| `(res T E)` | `(res.ok x)` or `(res.err e)` |
| `(map K V)` | hash map; each call passes `(fn ((& K)) i64)` and `(fn ((& K) (& K)) bool)`. Helpers cover `str` and `i64` |

## Function types

`(fn (A B) R)`: pointer to a non-capturing function. Named functions coerce to it.

## Inference

Bidirectional, local to a function body. Expected type flows into literals, `opt.none`,
`vec.new`, and generic calls. Unsuffixed integer literal with no expected type is `i64`.
No global inference; every `fn` signature is fully annotated.

## Arithmetic

`+` `-` `*` and `neg` wrap in two's complement. `llrc build --checked` makes those four trap
with `P|FILE|LINE|COL|int.overflow` at the expression, and does the same for signed
`MIN / -1` and `MIN % -1`. `wadd` `wsub` `wmul` `wneg` wrap either way. Shifts and casts
are not checked.

## Casts

`(as T e)` between numeric types only; truncating/sign-extending/float-int by Rust rules.
No implicit numeric conversion anywhere.

## Traits (v1)

Only compiler-known traits exist: `drop` (see memory.md), `copy` (primitives, refs, raw
pointers, `(fn ...)`). User traits are not in v1; use generic functions and enums instead.
