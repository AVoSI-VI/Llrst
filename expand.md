# expand: fixed arrays, i128 and u128, and fixed storage in std and self

This file is the work plan for widening the Llrst language. It is written so another agent can
pick it up cold. Read [HANDOFF.md](HANDOFF.md) first (rules, architecture, how to verify), then
this file, then `docs/canon/README.md`. [langspeed.md](langspeed.md) is the sibling plan for speed;
its benchmark gate (`benchmarks/run.sh --check`) applies to every phase here that touches a hot
`std/` file or `self/cg_*.llr`. When code and this file disagree, trust the code and fix this file.

Status legend: `[ ]` not started, `[~]` in progress, `[x]` done. Update the boxes as you go and
add a dated line to the **Progress log** at the bottom.

Scope, decided 2026-11-09:

- Fixed arrays `(arr T N)` as a language type, implemented in stage 0 (`compiler/`, Rust on
  Cranelift) and then in `self/` for x86-64, aarch64, riscv64, and wasm, so stage 2 is equal.
- `i128` and `u128`. The `i128` plan that was Phase 21 of `HANDOFF.md` lives here now (Phase B)
  and is extended to `u128`. `HANDOFF.md` Phase 22 (TLS) depends on Phase B.
- A measured pass over `std/` and `self/` replacing `vec` with `(arr T N)` where the length is a
  constant and the vec is allocated per call or per block (Phase C). Candidates are listed under
  **Where vec is fixed-size in practice** with evidence; each is promoted or killed by a number.
- Every phase ends with the `HANDOFF.md` bar: `tests/run.sh` fails=0 under stage 0 and under
  stage 2 (`scripts/self.sh` reaches `self|stage2=equal`), `tests/check.sh` and `tests/mir.sh`
  byte-equal, `scripts/check-lines.sh` clean, docs updated in the same commit.

Non-goals: const generics (`<T N>` on a user fn), slices, tuples, `usize`, array trait impls
(`Eq Ord Hash Clone Show` on `(arr T N)` need const generics), array patterns, array `const`s
(`(const ...)` stays literal-only), and any change that exists only to make `self/` smaller.

## Rules for this work

Those in `HANDOFF.md` **Rules for this work**, plus:

- Stage 0 first, then `self/`. `self/` may use a new type in its own source only after the stage
  0 commit that compiles it has landed and `scripts/self.sh` has rebuilt `llrs0` from it.
- Both compilers desugar the same way. A new surface form is checked into the same THIR shape and
  lowered to the same MIR text in `compiler/src/` and `self/`; `tests/check.sh` and `tests/mir.sh`
  are the proof. Add the MIR goldens to `tests/mir.sh`'s input set when it enumerates files.
- One new error code is one row in `docs/spec/errors.md`, one `tests/err` case, and one line in
  `docs/workflows/debug.md` if that file maps codes to actions.
- Nothing in Phase C lands without a `benchmarks/` line or an `llr-bench` micro that moved. A
  replacement that does not move a number is reverted, and the row is marked killed here.

## Design: fixed arrays

Surface, in `docs/spec/grammar.md` terms:

```
TYPE  += (arr TYPE N)                      ; N is a decimal integer atom, N >= 1, no suffix, no _
EXPR  += (arr.of EXPR EXPR...)             ; (arr T N) from N >= 1 elements; T from want or the first element
        | (arr.filled EXPR)                ; every element a copy of EXPR; the type comes from want
        | (arr.filled <(arr T N)> EXPR)    ; same, type given explicitly
        | (arr.len (& a))                  ; i64 N, known at compile time; the argument is still evaluated
        | (arr.get (& a) i)                ; (& T), panics oob like vec.get
        | (arr.get_mut (&mut a) i)         ; (&mut T)
        | (arr.set (&mut a) i x)           ; unit; drops the old element
```

| topic | decision |
| --- | --- |
| type | `Ty::Arr(Box<Ty>, u64)` in stage 0 (`compiler/src/ty.rs`). In `self/ty.llr` it is `sty` kind 16 with `id` = N and `args[0]` = T; `hash` and `eq` already fold `id` and `args`, so `(arr u8 4)` and `(arr u8 8)` differ with no struct change. Shown as `(arr i64 4)` by both printers. |
| parse | No parser change. `(arr i64 4)` parses today as `TyExpr::App("arr", [Name("i64"), Name("4")])` (`compiler/src/parse/ty.rs:70-73`) and as `std/ast.llr` kind 1 app with two kind-0 names. `resolve_ty_body` (`compiler/src/resolve.rs:421`, next to the `box` arm) and `self/resolve.llr` / `self/collect.llr:333 resolve_tys` turn the head `arr` into the type. A second argument that is not a decimal atom `>= 1` is `type.arr_len\|got=`. Wrong arity is the existing `type.generic_arity\|name=arr`. |
| layout | size `N * size(T)`, align `align(T)`, element `k` at `k * size(T)`. `Layout.fields` stays empty; `Rvalue::Struct` codegen computes `k * size` for an `Arr` destination instead of reading `fields[k]` (`compiler/src/codegen/func.rs:340-348`, `self/cg_term.llr` / `self/wasm_fn.llr store_agg` equivalents). `(sizeof (arr u8 16))` is 16. |
| value | `copy` iff `T` is `copy` (`Ty::is_copy`, `self/ty.is_copy`). Moves as a whole. `needs_drop` iff `T` does. Drop glue is one counted loop over `N` calling `T`'s glue (`compiler/src/codegen/drop.rs` new `drop_arr`; `self/cg_drop.glue` and `self/wasm_drop.glue` kind 16). The glue frame name is `drop.(arr T N)` through the type printer, same as structs. |
| ABI | An array is an aggregate (`is_agg`): passed by pointer and copied into the callee slot, returned through sret, exactly the struct path. `cg_frame`, `cg_term`, `cg_clos`, and the `*_abi.llr` files are untouched. An `(arr ...)` in an `extern` signature is `type.extern_ty\|ty=`, the one code Phase B also uses for `i128` and `u128` (it replaces the `type.extern_wide` name the old Phase 21 proposed). |
| MIR | `Proj::Index(LocalIdx)` on places (`compiler/src/mir/mod.rs:30`; `self/mir.llr` place kind 3 with `pi` = the index local). Printed `_a[_i]`, so `(*_3)[_4]` for a deref base. The index is always a local: the checker binds it first. `Index` appears only as the last projection, and only in `Rvalue::Ref`, `Stmt::Drop`, and the destination of `Stmt::Assign` (`arr.set`, `arr.filled`). An `Operand` never carries `Index`: element reads go through the reference `arr.get` returns. |
| checker desugar | `(arr.get A I)` becomes `(do (let _a A) (let _i I) (if (or (< _i 0) (>= _i N)) (panic (& (str.oob N _i)))) (& (index (@ _a) _i)))` in THIR, with a new place node `ExprKind::Index { base, idx }` and the existing `panic` lowering to `sys.panic(loc, m)` (`loc` is the call's span through `loc_lit`). `arr.get_mut` is the same with `&mut`. `arr.set` evaluates `A`, `I`, the check, then `X`, and ends in `(set (index (@ _a) _i) X)`, which drops the old value by the existing `set` rule. `arr.of` is `ExprKind::Arr(Vec<Expr>)` lowered to `Rvalue::Struct((arr T N), ops)`. `arr.filled` is `(do (let mut _x X) (let mut _a (arr T N) <empty arr>) (for _k 0 N (set (index _a _k) _x)) _a)` where `<empty arr>` is `Arr([])` lowered to `Rvalue::Struct((arr T N), [])`, a whole-local write that initializes `_a` for `moves.rs` and sets its drop flag; `X` must be `copy`, else `type.arr_fill\|ty=`. `arr.len` is `(do A N)` with `N` an `i64` literal. The two-compare form is kept over `(< (as u64 _i) N)` because `--checked` would turn a negative index into `cast.overflow`. |
| borrowck | `overlaps` (`compiler/src/borrow/mod.rs:128`, `self/borrow.llr`) treats any two `Index` projections as equal: a loan on `a[i]` conflicts with every access to `a[j]` and to `a`, and `a[i].0` is still disjoint from `a[j].1`. Every place walk adds a `Read` access of each `Index` local (`stmt_accesses`, `term_accesses`, `self/borrow.llr` accesses) so liveness, `init.uninit`, and `name.unused` see the index. `holders`: an index local is an integer and holds nothing. No new move rule: a by-value element read is `(@ (arr.get ...))`, already `move.deref` for a non-copy `T`. |
| optimizers | `self/optfwd.llr` does not forward a local that appears as an `Index` (it has no operand slot to receive a constant); `uses` counts the index as a use. `self/inline.llr` and `compiler/src/mir/inline.rs` rename `Index` locals with the rest of the place. `cg_slot.note` already sends a local with any projection to memory, so the array lives in its slot and the index local may stay in a register. |
| codegen place | `place_addr` / `self/cg_place.step` kind 3: load the index local as an 8-byte scalar into a scratch register, multiply by `size(T)` (`imul` by immediate, or a shift when the size is a power of two, through a `cg_isa` op; check `cg_op` for the multiply the `*` operator already uses), add to `cg_isa.addr`. The scratch register must not be `addr` or `acc`. Stage 0: `imul_imm` then `iadd`, mirroring `Rvalue::PtrAdd` (`func.rs:431-438`). wasm: the same on the linear-memory address in `self/wasm_op.llr` / `wasm_abi.llr` place code. |
| pointer | No `arr.ptr`. `(as (ptr T) (& a))` is already a legal cast (`compiler/src/check/expr.rs:174`, `(Ref, Ptr) => true`) and is the one way to hand an array to an extern. `docs/canon/collections.md` says so. |
| not in | patterns (`match.not_enum`), trait impls, `vec` to and from `arr`, `arr.eq`, `arr.clone`, `arr.ptr`, a cap on `N`. A stack array larger than the thread stack fails at runtime the way deep recursion does. |

## Design: i128 and u128

Everything the old Phase 21 decided stays, with `u128` added as int id 9:

| topic | decision |
| --- | --- |
| ids and names | Stage 0 `IntTy::I128`, `IntTy::U128`. `self/ty.llr` ids 8 and 9; `signed` is `(or (< i 4) (== i 8))`; `int_bytes` 16 for both. Fold `cg_lay.int_bytes`, `cg_bits.int_w`, and `cg_lay.signed` into `ty.int_bytes` and `ty.signed` (three copies today). |
| layout and ABI | 16 bytes, aligned 8 (no C interop; every slot is 8-aligned). In `self/` both are `cg_lay.is_agg`, not `is_scalar`: struct path for params, slots, and returns. In stage 0 `layout::clif_type` maps them to `types::I128` (fold the private duplicate `ovf.rs:int_clif` into it). An `i128` or `u128` in an `extern` signature is `type.extern_ty\|ty=`. |
| literals | Stage 0 keeps `Lit::Int(i128, suffix)` and `Const::Int(i128, IntTy)`; a `u128` value is stored as its two's-complement bit pattern. The parser (`compiler/src/parse/expr.rs:104-109`) falls back to `u128::from_str_radix` when the `i128` parse fails and the atom is not negative, producing a new `Lit::Wide(u128, suffix)`; the checker accepts `Lit::Wide` only for `u128` (`lit.overflow\|ty=` otherwise) and lowers it to `Const::Int(v as i128, U128)`. `fits(U128, v)` is `v >= 0` for `Lit::Int` and always true for `Lit::Wide`; `fits(I128, _)` is true; neither evaluates `1i128 << 128`. `mir/print.rs` prints a `U128` constant as `v as u128` so `340282366920938463463374607431768211455u128` round-trips. `self/lit.num` gains `(wide bool)`; `val` becomes `i128`, and `parse_dec` / `parse_rad` overflow-check against `MAX / base` before each step instead of `(< v 0)`. `thir.ex.id2` becomes `i128`; `mir.op` gains `hi i64` so `oconst` carries 128 bits; `mir_print.int_txt` prints through `Show.show` on `i128` or `u128` by type. |
| arithmetic | `+ - * neg` wrap. `/` and `%` lower to `sys.div128` (signed) or `sys.udiv128` (unsigned) with no widening (`lower_div.rs`, `self/lower_div.llr into_div` skip `widen` for ids 8 and 9). Under `--checked`, `*` lowers to `sys.checked_mul128` or `sys.checked_umul128`; add, sub, and neg trap from the sign bits (signed) or the carry (unsigned); shifts mask with 127 or trap outside `0..128`. No backend emits a 128-bit divide or multiply-overflow. |
| casts | No float cast to or from a 128-bit int (`type.bad_cast`; go through `i64`). Int-to-int: widen by sign- or zero-fill of the high word; narrow takes the low word and, under `--checked`, traps `cast.overflow` unless the high word is the sign fill (signed target) or zero (unsigned target). `i128` to `u128` and back is a bit copy; under `--checked` it traps when the sign bit is set. |
| compare | Signed compares use the signed high-word compare; `u128` uses unsigned on both words. The low word is always unsigned. |
| patterns | A 128-bit literal is not a `match` pattern (`match.bad_lit`). `Term::Switch` keeps its `u128` case values. |
| std | `std/sys.llr`: `udiv128 ((loc (& str)) (a u128) (b u128) (rem bool)) u128` by shift-subtract using only `shl shr - < ==`, `div.zero`; `div128 ((loc) (a i128) (b i128) (rem bool)) i128` takes magnitudes, calls `udiv128`, fixes signs, `MIN / -1` and `MIN % -1` are `int.overflow`; both keep the `raw` exemption `checked_div` has so their own `/` does not recurse. `checked_mul128` and `checked_umul128` multiply wrapping, then divide back to detect overflow (`MIN * -1` for signed). `std/cmp.llr`: `Eq Ord Hash Clone Show` for `i128` and `u128` mirroring the `u64` block; `Hash` folds the two 64-bit halves with `bxor` before the `i64` mix; `Show u128` is a digit loop over `udiv128`, `Show i128` handles the sign and `MIN` then calls it. `assert_eq` reaches `test.eq_by` through `Show` and `Eq`. |
| stage 0 codegen | `codegen/wide.rs` (new): constants are `iconcat` of two `iconst` I64 (`func.rs:289` and `term.rs:123` stop truncating with `as i64`); `icmp_imm` on I128 becomes `icmp` against a built constant; `--checked` add, sub, and neg derive overflow from the sign bits with `bxor band ushr 127` (signed) or `icmp ult` of the sum against an operand (unsigned), because Cranelift 0.136 x64 does not lower `*_overflow`, `sdiv`, `udiv`, `clz`, or `fcvt_*` on I128 (the old Phase 21's finding: confirm on the first build with a one-file experiment before relying on it); `sextend uextend ireduce` for int casts; `mask_shift` masks with 127, `checked_shift` traps outside `0..128`. |
| self codegen | `cg_wide.llr` (new) evaluates every 128-bit op on two 8-byte halves in memory through new `cg_isa.llr` seam ops `add_wide sub_wide mul_wide neg_wide not_wide and_wide or_wide xor_wide shl_wide shr_wide sar_wide cmp_wide ucmp_wide ext_wide zext_wide trunc_wide`. `x64_wide.llr` encodes `adc sbb shld shrd` and one-operand `mul`/`imul` (`rdx:rax`); `mul_wide` is `mul lo*lo` plus two `imul` cross terms. `a64_wide.llr` uses `adds adcs subs sbcs` (new encodings) and the existing `mul umulh madd`. `rv_wide.llr` uses `add sltu` for carry and `mul mulhu mulh` (RV64M). `wasm_wide.llr` holds a value as two `i64` words in linear memory; carry is `i64.lt_u` of the low sum; `mul_wide` builds the high word from four 32-bit products; shifts branch on `count >= 64`. `wasm_op.llr`, `wasm_abi.llr`, and `isa_a64.llr` only dispatch to the new files. |

## Where vec is fixed-size in practice

Read from `std/`, `self/`, `ui/`, and `benchmarks/` on 2026-11-09. Nothing here was timed yet;
Phase C measures each promoted row before and after with the stated bench and writes the number
into the **verdict** column. A row is killed when the length is not a constant, when the vec feeds a
growable byte stream, or when the fix is something other than an array.

| site | evidence | what an array buys | verdict |
| --- | --- | --- | --- |
| SHA-1 message schedule | `std/sha.llr:28-37 schedule` builds a `(vec u32)` of 80 per 64-byte block; `be32` at `:23-26` allocates a 4-byte vec per word read (16 per block); `push32` at `:57-60` a 4-byte vec per output word | `(arr u32 80)` on the stack and reading bytes from the `(& str)` directly remove 18 allocations per block | promote; bench: `llr-bench` micro over `sha.sha1` of 64 KiB, add to `benchmarks/micro/` |
| SHA-256 schedule and state | `std/sha256.llr:42-52 sched` copies the chunk with `bytes.from_str` and builds a 64-entry `(vec u32)` per block; `init` at `:30-40` and `compress (h (&mut (vec u32)))` hold the 8-word state in a vec | `(arr u32 64)` schedule, `(arr u32 8)` state, no chunk copy | promote; same micro as above over `sha256`. Phase 22 of `HANDOFF.md` (TLS) runs this per record |
| `Show u64` digit buffer | `std/cmp.llr:153-164 show_u` pushes digits into a `(vec u8)` then copies them into the `str` | `(arr u8 20)`, and `(arr u8 39)` for the 128-bit `Show` added in Phase B | promote; micro: `Show.show` of a `u64` in a loop |
| `self/` MIR places | `self/mir.llr:7 place` holds `pk (vec u8)` and `pi (vec i64)`; `ppush` allocates both, `pclone` and `oclone` copy both, on every projected place | none without a cap on projection depth, and a cap is a language limit (`(. (. (. a b) c) d)` is unbounded) | killed; do not re-propose. The right fix is an interned place table, which is a `langspeed.md` item, not an array |
| `std/map.llr:11-14` heads | 16 initial heads, then growth | none; the length is not a constant | killed |
| borrow bitsets | `self/borrow.llr:14-27`, `holders.llr`, `liveness.llr` size words by loan and local counts | none | killed |
| code buffers | `self/x64_buf.llr`, `a64_buf.llr`, `rv_buf.llr`, `wasm_*` byte vecs grow with the function | none | killed |
| allocator directory | `std/alloc.llr:76` is already one raw block of 14 heads | none | killed |
| fannkuch permutation | `benchmarks/tasks/fannkuch/fannkuch.llr:10-11 flips` clones a 9-element `(vec i64)` per call; `langspeed.md` names this as the gap | `(arr i64 9)` is a 72-byte copy with no allocation | measure only after `docs/canon/collections.md` lists `arr`; `langspeed.md` forbids a change that helps one task, so this lands only if the canon file recommends `arr` for small fixed tables generally |
| a64 ABI class table | `self/a64_abi.llr:71-76 tail` builds a fixed 18-entry vec | none; it is test scaffolding | killed |

Candidates to look for while doing Phase C, not yet found with evidence: an `(arr u8 N)` stack
buffer for `read` in `std/io.llr:13 read_fd` (today it reserves into the result `str` directly, so
probably nothing), and small per-call tables in `std/re.llr`, `std/json_read.llr`, and
`ui/raster*.llr` `zeros` helpers (pixel buffers are not constant-sized; likely killed).

## Phases

Each item is its own commit where the repo already works that way. The global bar never moves:
`tests/run.sh` fails=0 under stage 0 and under stage 2, `tests/check.sh` and `tests/mir.sh`
byte-equal, `benchmarks/run.sh --check` when a hot file or `self/cg_*.llr` changes.

### Phase A0: arrays, spec and stage 0 front end `[x]`

Files: `compiler/src/ty.rs`, `compiler/src/resolve.rs`, `compiler/src/check/zonk.rs`, `compiler/src/check/expr.rs`, `compiler/src/check/call.rs`, `compiler/src/thir.rs`, `compiler/src/mir/mod.rs`, `compiler/src/mir/lower.rs`, `compiler/src/mir/lower_call.rs`, `compiler/src/mir/print.rs`, `compiler/src/mir/inline.rs`, `compiler/src/mir/lint.rs`, `compiler/src/mir/speed.rs`, `compiler/src/borrow/mod.rs`, `compiler/src/borrow/liveness.rs`, `compiler/src/borrow/holders.rs`, `compiler/src/borrow/moves.rs`; `docs/spec/types.md`, `docs/spec/grammar.md`, `docs/spec/tokens.md`, `docs/spec/errors.md`, `docs/spec/memory.md`, `docs/canon/collections.md`, `tests/err`.

1. `[x]` Write the spec first: the `arr` row in `docs/spec/types.md` (size, align, copy rule, aggregate ABI, drop loop), the grammar lines above in `docs/spec/grammar.md`, `arr` in `docs/spec/tokens.md` keywords, the `## arr` table in `docs/canon/collections.md` (create, fill, length, read, read mutable, overwrite, pointer for an extern, iterate with `for`; "no `Eq`, `Clone`, or `Show`: compare and copy by element"), the loan rule sentence in `docs/spec/memory.md`, and `type.arr_len`, `type.arr_fill`, `type.extern_ty` rows in `docs/spec/errors.md`.
2. `[x]` `Ty::Arr` in `ty.rs`: `is_copy` (element), `has_ref`, `has_infer`, `has_param`, `subst`, `lifetimes`, `TyPrinter::show` as `(arr T N)`. `resolve.rs:421` gets the `arr` arm. `zonk.rs` unifies `Arr` on equal `N` and the element. `check/expr.rs` is at 427 lines and `mir/lower.rs` at 451: split before adding (`check/arr.rs` for the desugars, `mir/lower_place.rs` for place lowering).
3. `[x]` THIR: `ExprKind::Index { base, idx }` (a place when `base` is a place) and `ExprKind::Arr(Vec<Expr>)`. `thir::Builtin` gains `ArrOf ArrFilled ArrLen ArrGet ArrGetMut ArrSet` as dispatch keys for `Builtin::from_path`; `check_builtin` produces the desugars in the **Design** table and never returns a `Callee::Builtin` for them. `arr.filled` reads its type from `targs[0]` or `want`; neither known is `type.infer\|fix=(let NAME (arr T N) ...)`.
4. `[x]` MIR: `Proj::Index(LocalIdx)`; `place_ty` returns the element type; `print.rs` prints `_a[_i]`; `Rvalue::Struct` print is unchanged and reads `(arr i64 3) {1i64, 2i64, 3i64}`. Place lowering requires `idx` to be `ExprKind::Local`. `inline.rs` renames the index local. `lint.rs` counts it as a use. `mir/speed.rs` place walkers get the arm (`rg "Proj::" compiler/src` lists every site: `mir/mod.rs`, `mir/print.rs`, `mir/lower_match.rs`, `borrow/mod.rs`, `borrow/moves.rs`, `codegen/func.rs`).
5. `[x]` Borrowck: `overlaps` equates any two `Index`; `stmt_accesses` and `term_accesses` push a `Read` for each `Index` local via one helper; `liveness.rs` and `holders.rs` see the read through that helper. `moves.rs` needs no new rule; confirm `init.uninit` fires for an uninitialized index local with a `tests/err` case.
6. `[x]` `tests/err`: `arr_len` (`(arr i64 0)`, `(arr i64 x)`, `(arr i64 4u8)`), `arr_fill` (`(arr.filled (str.from "x"))` with a known type), `arr_extern`, `arr_infer` (`(let a (arr.filled 0))`), `arr_escape` (a reference from `arr.get` outliving the array: `borrow.escape`), `arr_mut_shared` (`arr.get` held across `arr.set`: `borrow.mut_shared`), `arr_uninit` (an uninitialized index local: `init.uninit`). `llrc check` goldens only; codegen is A1. `tests/check.sh` and `tests/check.ps1` skip `arr_*` until A2.
7. `[x]` Commit: `lang: (arr T N) type, arr.* desugars, Index places in stage 0 front end`.

### Phase A1: arrays, stage 0 codegen and goldens `[x]`

Files: `compiler/src/codegen/layout.rs`, `compiler/src/codegen/func.rs`, `compiler/src/codegen/place.rs`, `compiler/src/codegen/drop.rs`, `compiler/src/codegen/mod.rs`, `compiler/src/check/arr.rs`; `tests/run`, `tests/panic`, `tests/mir.sh`, `tests/mir.ps1`.

1. `[x]` `layout.rs`: `Ty::Arr` size and align (landed in A0); `clif_type` returns `None` (aggregate, already the `_` arm). `func.rs place_addr` `Index` arm still panics: load the index local (`local_addr` then `load` as I64; a register-resident Cranelift value is fine), `imul_imm` by `size(T)`, `iadd`. `assign` `Rvalue::Struct` uses `k * size` for an `Arr` destination. `func.rs` is at 461 lines: move `place_addr` and the load/store helpers into `codegen/place.rs` first.
2. `[x]` `drop.rs drop_arr`: counter in a block parameter, body calls the element glue at `addr + k * size`, exits at `N`. The glue function name comes from `drop_frame` through `shown`, so it is `drop.(arr str 2)`.
3. `[x]` Goldens: `tests/run/arr.llr` (`arr.of` of `i64`, `u8`, `f64`, `str`, and a struct; `arr.filled` with `want` and with `<(arr u8 64)>`; `arr.len`; `get` `get_mut` `set`; a `for` over it; an array field in a struct, an array element of a `vec`, an array of arrays; pass by value, return by value, `(let b a)` copy for `(arr i64 4)` and move for `(arr str 2)`; `(sizeof (arr u8 16))`; `(as (ptr u8) (& buf))` handed to `sys.write_all`; drop order of `(arr str 3)` observed through a `drop` method on a wrapper struct); `tests/panic/arr_oob arr_oob_neg arr_set_oob` with the `P|...|index` record matching `vec.get`'s shape; `tests/run/arr_checked.llr` with a `.flags` file of `--checked`. `emit-mir` of `tests/run/arr.llr` is the MIR golden `tests/mir.sh` will compare against stage 2 in A2.
4. `[x]` Commit: `lang: arrays in stage 0 codegen, goldens`.

### Phase A2: arrays, self front end `[x]`

Files: `self/ty.llr`, `self/resolve.llr`, `self/collect.llr`, `self/thir.llr`, `self/thir_expr.llr`, `self/thir_call.llr`, `self/lower.llr`, `self/lower_call.llr`, `self/mir.llr`, `self/mir_print.llr`, `self/inline.llr`, `self/lint.llr`, `self/optmir.llr`, `self/optfwd.llr`, `self/borrow.llr`, `self/loans.llr`, `self/holders.llr`, `self/liveness.llr`, `self/moves.llr`, `self/static_ck.llr`, `self/cg_slot.llr`.

1. `[x]` `ty.llr`: kind 16 in `is_copy` (element), `has_ref`, `has_infer`, `lts`, `subst_one` (map `args`), `show` as `(arr T N)`; `fits` untouched. `thir.unify` kind 16: equal `id`, unify `args[0]`. `resolve.llr` / `collect.llr resolve_tys`: head `arr` with the same `type.arr_len` text as stage 0.
2. `[x]` `thir_call.llr` (432 lines: split a `thir_arr.llr` first) produces the same desugars as `check/arr.rs`, including the `loc` literal text, the temp names, and the `or` shape, so `emit-mir` is byte-equal. `thir.ex` gains the index and array node kinds; note the kind numbers in the file-head comment of `std/ast.llr` style.
3. `[x]` `mir.llr`: place kind 3; `place_ty`; `has_deref` and `lone_field` unchanged; `mir_print.place_of` prints `[_i]`. Every file in `rg -l "pk" self` that walks projection kinds gets the arm: `lower*.llr`, `inline.llr`, `lint.llr`, `optmir.llr`, `optfwd.llr` (no forwarding into an index local), `borrow.llr` (`overlaps`, index reads), `loans.llr`, `holders.llr`, `liveness.llr`, `moves.llr`, `static_ck.llr`, `cg_slot.llr` (already sends a projected local to memory; confirm with the test in that file).
4. `[x]` Drop the `arr_*` skip in `tests/check.sh` and `tests/check.ps1`, and the `arr` / `arr_checked` skip in `tests/mir.sh` and `tests/mir.ps1`. `tests/check.sh` and `tests/mir.sh` fails=0 with `tests/run/arr.llr`, `tests/run/arr_checked.llr`, and the `tests/err/arr_*` cases included. `scripts/self.sh --llrs0` builds.
5. `[x]` Commit: `self: arrays in check, lower, borrow; check and emit-mir byte-equal`.

### Phase A3: arrays, self codegen, stage 2 equal `[ ]`

Files: `self/cg_lay.llr`, `self/cg_place.llr`, `self/cg_term.llr`, `self/cg_drop.llr`, `self/cg_isa.llr` (only if a multiply-by-immediate seam op is missing), `self/wasm_op.llr`, `self/wasm_abi.llr`, `self/wasm_fn.llr`, `self/wasm_drop.llr`; `docs/spec/targets.md`.

1. `[ ]` `cg_lay.compute` kind 16: `scalar`-shaped `lay` with `size = N * size(T)` and `align(T)`, empty `fields`. `is_scalar` stays false, so `is_agg` is true.
2. `[ ]` `cg_place.step` kind 3 per the **Design** table. The index local is read with the same path `cg_op` uses for a plain-local right operand (register or `fp+disp`), into a register that is neither `cg_isa.addr` nor `cg_isa.acc`. The `Rvalue::Struct` store path in `cg_term.llr` computes `k * size` for an array destination.
3. `[ ]` `cg_drop.glue` kind 16 `glue_arr`: a loop with the counter in a callee-saved register the glue already saves, calling the element glue per step through the path `field_one` uses. `cg_drop.llr` is at 206 lines; a new `cg_drop_arr.llr` is fine.
4. `[ ]` wasm: place address, `Struct` store, and `glue` arms in `wasm_op.llr` / `wasm_abi.llr` / `wasm_fn.llr store_agg` / `wasm_drop.llr`. `tests/wasm.sh` runs `arr` and `arr_checked`.
5. `[ ]` `scripts/self.sh` is `self|stage2=equal`; `tests/run.sh` fails=0 under `llrs2`; `tests/a64.sh` and `tests/cross.sh` for aarch64 and riscv64 when a Mac or qemu is available, else say so in the log. `docs/spec/targets.md` notes nothing new unless wasm needed a different form.
6. `[ ]` Commit: `self: arrays on x64, aarch64, riscv64, wasm; stage2=equal`.

### Phase B0: i128 and u128, stage 0 front end `[ ]`

Files: `compiler/src/ty.rs`, `compiler/src/ast.rs`, `compiler/src/parse/expr.rs`, `compiler/src/check/expr.rs`, `compiler/src/check/pat.rs`, `compiler/src/check/assert.rs`, `compiler/src/mir/lower.rs`, `compiler/src/mir/lower_div.rs`, `compiler/src/mir/print.rs`; `docs/spec/types.md`, `docs/spec/tokens.md`, `docs/spec/errors.md`, `tests/err`.

1. `[ ]` Spec first: `i128 | 16 | signed; aligned 8; no float cast; not an extern type` and the `u128` row in `docs/spec/types.md`, the division and `--checked` notes, both names and suffixes in `docs/spec/tokens.md`, `type.extern_ty` and the `match.bad_lit` and `type.bad_cast` cases in `docs/spec/errors.md`.
2. `[ ]` `IntTy::I128`, `IntTy::U128`: `from_name`, `name`, `bytes` 16, `signed`, `fits` per the **Design** table. `Lit::Wide` in `ast.rs` and `parse/expr.rs`; the four `Lit::Int` match sites (`check/expr.rs`, `check/pat.rs`, `parse/expr.rs`, `mir/lower.rs`) get the `Wide` arm. `check/expr.rs`: `(as ...)` between a 128-bit int and `f32`/`f64` is `type.bad_cast`; `check/pat.rs` rejects a 128-bit literal pattern as `match.bad_lit`; the extern signature check reports `type.extern_ty` (today an aggregate extern parameter is not diagnosed at all: find where `is_extern` params are walked and add the check for `Arr`, `I128`, `U128`, and any other aggregate). `lower_div.rs`: ids 8 and 9 call `sys.div128` / `sys.udiv128` with no widening; `--checked` `*` calls the matching `checked_*mul128`. `print.rs` prints `U128` unsigned.
3. `[ ]` `tests/err`: `i128_extern`, `i128_float_cast`, `i128_match`, `lit_overflow_i128`, `lit_overflow_u128` (a `Lit::Wide` with an `i128` want), `u128_neg_lit` (`-1u128`).
4. `[ ]` Commit: `lang: i128 and u128 in stage 0 front end`.

### Phase B1: i128 and u128, stage 0 codegen `[ ]`

Files: `compiler/src/codegen/layout.rs`, `compiler/src/codegen/func.rs`, `compiler/src/codegen/ops.rs`, `compiler/src/codegen/ovf.rs`, `compiler/src/codegen/term.rs`, new `compiler/src/codegen/wide.rs`.

1. `[ ]` One-file experiment first: build a Cranelift function with `iadd imul icmp ishl ushr sshr sextend uextend ireduce iconcat isplit` on `types::I128` for the host and for `aarch64`, `riscv64`, and `s390x` through `--target`. Record which lower. If one does not, route it through `isplit` into two I64 ops in `wide.rs` and say so in the **Design** table.
2. `[ ]` `wide.rs` per the **Design** table; `layout::clif_type` maps both ids to `types::I128` and `ovf.rs:int_clif` is folded into it; `func.rs:289` and `term.rs:123` stop truncating constants with `as i64`. `(sizeof i128)` is 16.
3. `[ ]` Goldens: `tests/run/i128.llr` and `tests/run/u128.llr` (literals in every radix including `Lit::Wide`, `MAX`, wrap at `2^127` and `2^128`, `+ - * neg bnot band bor bxor shl shr`, every compare including a `u128` above `2^127` against a small value, `/` and `%` with negatives, casts from and to `i8 u8 i64 u64` and between `i128` and `u128`, `(sizeof ...)` 16, a struct field and a `vec` element); `tests/run/i128_checked.llr` with `--checked` where every op fits; `tests/panic/i128_overflow i128_div_zero i128_div_min i128_mul_overflow i128_cast_high i128_shift_high u128_sub_underflow u128_cast_sign` with `--checked` where needed. `Show` and `assert_eq` goldens wait for B2.
4. `[ ]` Commit: `lang: i128 and u128 in stage 0 codegen, goldens`.

### Phase B2: std for 128-bit ints `[ ]`

Files: `std/sys.llr`, `std/cmp.llr`, `docs/std/README.md` (regenerated by `llr-doc`), `docs/canon/collections.md:5`.

1. `[ ]` `sys.udiv128`, `sys.div128`, `sys.checked_umul128`, `sys.checked_mul128` as in the **Design** table, each with a `(test ...)` form. `std/sys.llr` is small; the four fit under the 500-line cap.
2. `[ ]` `std/cmp.llr` is at 362 lines: the eight impls and `show_u128` go in a new `std/cmp_wide.llr` registered the way `str_more.llr` extends `str`. Tests in `cmp_ints` style for `(shl 1i128 100)`, `MIN`, `MAX`, negative `Show`, `u128` `MAX` `Show`, `Hash` of two values that differ only in the high word.
3. `[ ]` Extend `tests/run/i128.llr` and `u128.llr` with `Show` and `assert_eq`. `llr-doc` regenerates `docs/std`.
4. `[ ]` Commit: `std: div128, udiv128, checked muls, cmp for i128 and u128`.

### Phase B3: i128 and u128, self front end `[ ]`

Files: `self/ty.llr`, `self/lit.llr`, `self/thir_lit.llr`, `self/thir_expr.llr`, `self/thir_eq.llr`, `self/thir_pat.llr`, `self/lower.llr`, `self/lower_div.llr`, `self/mir.llr`, `self/mir_print.llr`, `self/cg_lay.llr`, `self/cg_bits.llr`, `self/collect.llr`, new `self/thir_ty.llr`.

1. `[ ]` `scripts/self.sh --llrs0` from the B2 tree so `self/` may use `i128` and `u128` in its own source. Ids 8 and 9 in `ty.llr`; fold the three copies of `int_bytes` / `signed` into `ty.llr`. `thir_expr.llr` is at 387 lines: `atom_ty`, `named_ty`, `check_cast`, and `cast_ok` move to `self/thir_ty.llr` so the float rule has room. `thir_pat.pat_lit` adds `match.bad_lit`; `collect.llr` adds `type.extern_ty`.
2. `[ ]` `lit.num` per the **Design** table; `thir.ex.id2` and `mir.op` carry 128 bits; `mir_print.int_txt` prints through `Show.show` by type. `lower_div.into_div` skips `widen` for ids 8 and 9.
3. `[ ]` `tests/check.sh` and `tests/mir.sh` fails=0 over the B0 and B1 goldens.
4. `[ ]` Commit: `self: i128 and u128 front end; check and emit-mir byte-equal`.

### Phase B4: i128 and u128 on x64, aarch64, riscv64 `[ ]`

Files: `self/cg_lay.llr`, `self/cg_op.llr`, `self/cg_cast.llr`, `self/cg_ovf.llr`, `self/cg_isa.llr`, new `self/cg_wide.llr`, new `self/x64_wide.llr`, new `self/a64_wide.llr`, new `self/rv_wide.llr`.

1. `[ ]` Both ids are `cg_lay.is_agg`. `cg_wide.llr` dispatches every binop, unop, compare, cast, and `--checked` trap for a 128-bit operand through the seam ops listed in the **Design** table; `cg_op`, `cg_cast`, and `cg_ovf` only test the width and call it. `isa_a64.llr` is at 321 lines and `isa_x64.llr` at 268: the encodings go in the `*_wide.llr` files, not there.
2. `[ ]` x64, then aarch64 (`tests/a64.sh` on a Mac or qemu), then riscv64 (`tests/cross.sh` plus qemu-riscv64 if present; otherwise record the untested ISA in the log and leave the row `[~]`).
3. `[ ]` `scripts/self.sh` is `self|stage2=equal`; `tests/run.sh` fails=0 under `llrs2`; `benchmarks/run.sh --check` because `cg_op.llr` changed.
4. `[ ]` Commit: `self: i128 and u128 on x64, aarch64, riscv64`.

### Phase B5: i128 and u128 on wasm `[ ]`

Files: new `self/wasm_wide.llr`, `self/wasm_op.llr`, `self/wasm_cast.llr`, `self/wasm_abi.llr`; `docs/spec/targets.md`.

1. `[ ]` `wasm_wide.llr` per the **Design** table; `wasm_op.llr` (404 lines) and `wasm_abi.llr` (352) only dispatch. `tests/wasm.sh` runs the B1 and B2 goldens. `docs/spec/targets.md` notes the two-word form.
2. `[ ]` Commit: `self: i128 and u128 on wasm`.

### Phase C: vec to arr where the length is a constant `[ ]`

Files: `std/sha.llr`, `std/sha256.llr`, `std/cmp.llr` or `std/cmp_wide.llr`, `benchmarks/micro/`, `benchmarks/baseline.txt`, `benchmarks/README.md`, `docs/canon/collections.md`.

Depends on A3 (and B2 for the 128-bit `Show`). One row per commit; the commit message names the bench line that moved.

1. `[ ]` Add `benchmarks/micro/sha1.llr`, `sha256.llr`, and `show_u64.llr` `(bench ...)` files and a first `baseline.txt` line for each on the unchanged tree (`benchmarks/run.sh --update-baseline`, same commit, says these are new lines).
2. `[ ]` `std/sha.llr`: `schedule` returns `(arr u32 80)`, `compress` takes `(& (arr u32 80))`, `be32` reads four bytes from the `(& str)` with `str.at`, `push32` pushes four bytes directly. Keep every existing `(test ...)`.
3. `[ ]` `std/sha256.llr`: `sched` returns `(arr u32 64)` read from the chunk with `str.at` (no `bytes.from_str`), state is `(arr u32 8)`.
4. `[ ]` `show_u` and `show_u128` write digits into an `(arr u8 20)` / `(arr u8 39)` and copy once.
5. `[ ]` Re-run `benchmarks/run.sh --check`; `--update-baseline` only for lines that moved, in the same commit. A row that did not move is reverted and marked killed in the table above with its numbers.
6. `[ ]` `docs/canon/collections.md` gains one sentence on when to pick `arr` over `vec` (constant length, built per call or per block, element type `copy`), and the `fannkuch` row above is decided from that sentence.
7. `[ ]` Commits: `std: sha1 schedule on the stack`, `std: sha256 schedule and state on the stack`, `std: show digits on the stack`, `bench: ...` as needed.

### Phase D: close out `[ ]`

1. `[ ]` `HANDOFF.md`: promote the two **Design** tables' decisions into **Decisions**; fill the **Status** `lines` and bench rows; move the `tuples, fixed arrays, slices, i128, usize` Gaps row to `tuples, slices, usize`; refresh **Line budget** for every file this plan split or created; one dated **Progress log** line per landed phase.
2. `[ ]` `docs/workflows/debug.md` rows for `type.arr_len`, `type.arr_fill`, `type.extern_ty`; `docs/spec/errors.md` already has them from A0 and B0.
3. `[ ]` Move this file to `history/expand.md` when every box is `[x]`, and point `HANDOFF.md`'s header list at it.

## How to verify each phase

```
cargo build --release                         # stage 0 llrc at target/release/llrc
tests/run.sh                                  # goldens + every std test + line check, stage 0
scripts/self.sh --stage                       # llrs0, llrs1, llrs2, stop at self|stage2=equal
scripts/self.sh                               # the same, then the suite under llrs2
tests/check.sh; tests/mir.sh                  # llrs check / emit-mir byte-equal to stage 0
tests/wasm.sh                                 # wasm goldens (A3, B5)
tests/a64.sh; tests/cross.sh                  # aarch64 and riscv64 (A3, B4), need qemu or a Mac
benchmarks/run.sh --check                     # after any hot std or self/cg_* change (A3, B4, C)
<llrc> emit-mir tests/run/arr.llr --std std   # read the Index places and the oob check shape
<llrc> check tests/err/X.llr --std std 2> tests/err/X.err   # regenerate a golden after a deliberate change
```

Windows twins: `tests/run.ps1`, `scripts/self.ps1`, `tests/check.ps1`, `tests/mir.ps1`, `tests/wasm.ps1`.

## Known unknowns (check before building on them)

- Cranelift 0.136 I128 lowering coverage is the old Phase 21's claim, not re-verified here. B1 item 1 is the experiment.
- Whether `compiler/src/mir/speed.rs` and `self/optmir.llr` walk places in a way that an `Index` projection can break (they match on `Proj`/`pk`; both are listed above, but the exact arms are not enumerated).
- The register `cg_place.step` can use as scratch for the index without clobbering a pending value. Read `cg_op.llr`'s right-operand path before choosing.
- Whether the empty `Rvalue::Struct((arr T N), [])` that initializes `arr.filled`'s local is accepted by every codegen path unchanged (stage 0 `assign` loops over zero operands; `self/cg_term.llr` and `wasm_fn.store_agg` may assume at least one).
- No aarch64 or riscv64 hardware was available when this plan was written; A3 and B4 say what to do when qemu is also missing.

## Progress log

- 2026-11-09: Plan written. Phase 21 (`i128`) of `HANDOFF.md` moved here as Phase B and extended to `u128`; `HANDOFF.md` Phase 22 now depends on this file's Phase B. The fixed-size vec table above was read from the code on this date and nothing in it has been timed.
- 2026-11-09: Phase A0. `(arr T N)` is a stage 0 type with `arr.*` desugars and `Index` places through check, MIR, and borrowck. `llrc check` goldens are `tests/err/arr_*`. Layout size and align are filled in; element addressing in codegen is A1. `tests/check.sh` skips `arr_*` until stage 2 matches. `inline.rs` renames an index local through `local_map` when it re-lowers THIR; it has no separate place walk. `mir/speed.rs` does not match `Proj`; a move of an indexed place does not clear the whole-local drop flag.
- 2026-11-09: Phase A1. Stage 0 addresses an index as `index * size(T)` (`codegen/place.rs`), stores an array `Rvalue::Struct` at `k * size`, and drops with a counted loop. The glue frame is `drop.(arr hold 2)` for an `(arr hold 2)`, via `shown`. An empty `(arr u8 64) {}` initializes `arr.filled`. `ref_arr` zonks before the `Arr` match so `arr.get` accepts the `(& T)` from `vec.get`. `tests/mir.sh` and `tests/mir.ps1` skip `arr` and `arr_checked` until A2; `tests/run.sh` under `llrs2` waits for A3.
- 2026-11-09: Phase A2. `self/` checks and lowers `(arr T N)` to the same MIR as stage 0. Kind 16 is `id` = N and `args[0]` = T; `has_infer` already walks `args`. THIR kinds are 36 index and 37 arr (`thir_arr.llr`). Place kind 3 prints `[_i]`. `borrow.push_pl` reads each index local before the place access, and any two index projections overlap. `optfwd` will not forward an index local; `optmir` will not clear a drop flag on an indexed move. `inline.llr` renames the index through `map_l` when it re-lowers THIR. `needs_drop` follows the element, including a struct field that is an array. `thir.keep` accepts a rendered resolve diagnostic so `type.arr_len` keeps the token span. `cg_slot` test `slot_index` confirms a kind-3 projection lives in memory. Codegen of the index is A3.
