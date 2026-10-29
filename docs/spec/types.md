# Types

## Primitives

| type | size | notes |
|------|------|-------|
| `i8 i16 i32 i64` | 1 2 4 8 | signed |
| `u8 u16 u32 u64` | 1 2 4 8 | unsigned |
| `f32 f64` | 4 8 | IEEE |
| `bool` | 1 | `true` `false` |
| `unit` | 0 | the only value is `()`; functions with nothing to return use it |
| `never` | 0 | no value; `os.exit` and `sys.panic` return it, so the caller does not continue |
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
| `(map K V)` | hash map. Key operations take `<K:Hash:Eq>` from `std/cmp.llr` |

## Function types

`(fn (A B) R)`: pointer to a non-capturing function. Named functions coerce to it.
`(clos (A B) R)` does not. A closure is an owned value: call code, drop code, and a heap
environment. Create one with `(clos ((n T)*) R BODY...)`. A `copy` binding is copied in;
anything else is moved in. A captured reference is a loan held by the closure. The value
can be returned and stored. Call it with `(call c a b)`. It is not `copy`, and a closure
body cannot contain a closure (`clos.nested`).

## Inference

Bidirectional, local to a function body. Expected type flows into literals, `opt.none`,
`vec.new`, and generic calls. Unsuffixed integer literal with no expected type is `i64`.
No global inference; every `fn` signature is fully annotated.

## Arithmetic

`+` `-` `*` and `neg` wrap in two's complement. Unchecked `shl` and `shr` mask the count
with `bitwidth - 1`, so a negative count uses the low bits of its two's-complement value.
`llrc build --checked` makes `+` `-` `*` and `neg` trap with `P|FILE|LINE|COL|int.overflow`
at the expression, and does the same for `shl` and `shr` whose count is outside `0..bitwidth`.
Signed `MIN / -1` and `MIN % -1` always panic `int.overflow`, with or without `--checked`.
`wadd` `wsub` `wmul` `wneg` wrap either way. An integer cast under `--checked` panics
`cast.overflow` when the value does not fit. A float-to-int cast panics the same way when
the truncated value does not fit, including `NaN` and infinities; otherwise it truncates
toward zero. An int-to-float cast panics when the integer is not exactly representable.
Without `--checked`, float-to-int still saturates and int-to-float still rounds.

## Casts

`(as T e)` between numeric types only; truncating/sign-extending/float-int by Rust rules.
No implicit numeric conversion anywhere. `llrc build --checked` panics with
`P|FILE|LINE|COL|cast.overflow` when an integer cast, a float-to-int cast, or an
int-to-float cast does not fit in `T`. Bool-to-int always fits. Without `--checked`,
float-to-int saturates and int-to-float rounds.

## Traits

`(trait Eq (fn eq ((& Self) (& Self)) bool))` declares a trait. `(impl Eq i64 (fn eq ...))` is the impl. The impl type is a concrete name. `(Eq.eq a b)` on a concrete type, including a substituted generic, is a direct call to that impl.

`<T:Eq:Ord>` adds bounds. A missing impl is `trait.missing|trait=|ty=`. A second impl is `trait.dup|trait=|ty=`.

`(& (dyn Eq))` and `(&mut (dyn Eq))` are a 16-byte value: data pointer, then vtable. Passing `(& n)` where that type is expected builds it when `n`'s type implements the trait. A method that returns `Self` or takes `Self` by value is `trait.object|why=self`, so `Clone` is static only.

`drop` and `copy` stay compiler-known. `copy` is primitives (including `bool` and `unit`), shared refs, raw pointers, `(fn ...)`, and a shared trait object.

## Memory and the host

`(sizeof T)` is the size in bytes of `T`, computed when the call is compiled. `mem.replace` swaps a place with a new value and returns the old one, including private fields of a whole struct. `mem.forget` drops the compiler's duty to run `drop` on a value. `sys.argc` and `sys.argv` read the process arguments. `sys.arch` is a compile-time `(& str)`.

Every local is a stack slot. Stage 0 sets Cranelift `opt_level=speed`. The installed `llrs` compiler does not.
