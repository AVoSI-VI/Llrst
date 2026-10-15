# Canon: the one right way

Each file lists tasks and the single blessed form. Forms marked `no` are rejected by
`llr-fmt --lint` with code `canon.<topic>.<rule>`.

| file | topics |
|------|--------|
| [program.md](program.md) | file layout, main, modules, imports |
| [bind.md](bind.md) | let, mut, set, shadowing |
| [flow.md](flow.md) | if, while, for, loop, match, early return |
| [errors.md](errors.md) | opt, res, propagation, panics |
| [strings.md](strings.md) | building, slicing, comparing, converting |
| [collections.md](collections.md) | vec, map iteration and mutation |
| [io.md](io.md) | stdout, stderr, files, args, exit codes |
| [tests.md](tests.md) | test forms, asserts, golden files |
| [refactor.md](refactor.md) | extracting functions, moving modules, renaming |

Rules that apply everywhere:

1. One form per task. If two forms compile, the canon one is the one here.
2. No comments except `;; why` for a non-obvious decision. Never `;; what`.
3. Files under 500 lines. Split by module at 400.
4. Functions under 40 lines. Extract at 30.
5. Names: lowercase, `_` separators, short.
6. Prefer `std` over new code. Prefer refactoring `std` over duplicating.
