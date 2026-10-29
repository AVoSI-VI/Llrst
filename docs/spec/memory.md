# Memory model

Rust rules, no annotations beyond `mut`, `&`, `&mut`, and rare `'a`.

## Ownership

1. Every value has exactly one owner (a local, a field, or a vec element).
2. Passing or assigning a non-`copy` value moves it. The source is unusable afterwards.
3. `copy` types: primitives (including `bool` and `unit`), shared references `(& T)`, raw
   pointers `(ptr T)`, `(fn ...)`, and a shared trait object `(& (dyn Trait))`. Structs and
   `(clos ...)` are never copy.
4. When an owner goes out of scope and still holds the value, `drop` runs. `str`, `vec`,
   and `map` free their heap memory. A closure drops each capture and frees its environment.
   Structs and enums drop their fields.
5. Drop order: reverse declaration order within a scope.
6. One field of a struct that has no `drop` method can be moved. That field is not dropped
   again; the other fields still are. A move through `&` or `ptr`, or out of a struct that
   has `drop`, is `move.field` or `move.deref`.

## Borrowing

1. `(& p)` creates a shared reference. Any number may coexist.
2. `(&mut p)` creates a unique reference. While it is live, the owner and all other
   references to any overlapping place are unusable.
3. A reference is live from creation to its last use (NLL). Last use is computed on the MIR
   control-flow graph, so a reference stops conflicting as soon as it is dead on every path.
4. Moving out of a borrowed place, or mutating a shared-borrowed place, is an error.
5. `(& (. p x))` borrows only field `x`; sibling fields stay usable.
6. Two-phase borrows: `(vec.push (&mut v) (vec.len (& v)))` is allowed; the `&mut` activates
   after the arguments are evaluated.
7. Reassigning a reference variable, or a struct holding references, as a whole
   (`(set r (& b))`, `(set v (view (& b)))`) releases the loans it held. Writing a single
   field (`(set (. v s) (& b))`) releases the loans stored in that field only.
8. Storing a reference into a container through `(&mut c)` (for example `vec.push`) makes
   the container a holder: the referent stays borrowed while `c` is live. Declare such a
   container after its referents so it is dropped first; otherwise `borrow.escape`.

## Lifetimes across functions

1. Elision: if a function has exactly one reference parameter, a returned reference derives
   from it. `(self ...)` first param counts as that one when present.
2. Otherwise, name lifetimes: `(fn pick <'a> ((a (& 'a str)) (b (& 'a str)) (c bool)) (& 'a str) ...)`.
   The returned reference derives from every argument sharing `'a`.
3. Struct fields may hold references: `(struct view <'a> (s (& 'a str)))`. The struct is live
   at most as long as the referents.
4. A returned reference derived from a local is an error `borrow.escape`.
5. A string literal is `(& str)` and points at static data. The name `'static` is that
   region for any type. A `'static` reference comes from a literal, a const, or another
   `'static` reference. Borrowing a `let` binding as `'static` is `borrow.escape|lt=static`.
   Any other named lifetime is only the cross-function flow in `eligible_args`.

## Raw memory

`(ptr T)` and `extern` are not checked. They exist so `std` can be written in Llrst.
Programs outside `std` should not use them; `llr-fmt --lint` flags them.

## Error codes emitted by the checker

| code | meaning |
|------|---------|
| `move.used` | use of moved value |
| `move.field` | move a field through `&` or `ptr`, or out of a struct that has `drop` |
| `move.deref` | move through a dereference |
| `move.borrowed` | move out of borrowed place |
| `borrow.mut_shared` | `&mut` while `&` live, or vice versa |
| `borrow.mut_mut` | two live `&mut` |
| `borrow.assign` | write to borrowed place |
| `borrow.escape` | reference outlives referent |
| `mut.required` | mutate or `&mut` a non-`mut` binding |
| `init.uninit` | use before every path assigns the local |
