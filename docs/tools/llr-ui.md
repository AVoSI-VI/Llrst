# llr-ui

Builds, serves, runs, dumps, or packs a Llrst UI.

```
llr-ui serve FILE [--out DIR] [--port N]
llr-ui pack --target TRIPLE FILE [--out DIR] [--sign IDENT]
llr-ui run FILE [--out DIR]
llr-ui dump FILE [--w N] [--h N] [--scale N] [--max N]
llr-ui shot FILE OUT.ppm [--w N] [--h N] [--scale N] [--frames N]
```

`FILE` is the program. The default out dir is `.llr-ui` and the default port is 0, which asks the kernel for a free port. `--max` defaults to 0, which means no limit. A cut list ends with `more|N`.

`serve` finds `llrc` on `PATH` and builds with `--target wasm32-unknown-unknown`. It copies `index.html`, `llr_ui.js`, and `ui/fonts/LiberationSans-Regular.ttf` into the out dir, then listens on `127.0.0.1`. The page polls `app.wasm`. When the bytes change it keeps the heap if the heap base is unchanged and no state `layout` is `-1`; otherwise it stores `save` in `sessionStorage` and loads it into a new instance.

`run` builds `FILE` with `--emit-dylib` and loads that library. The launcher owns the element and render arenas. It watches `FILE` and the `proj.*` and `ui.*` files it uses. A matching layout prints `run|reload` and re-runs `build` on the kept state. A changed layout, a closure in a state, or a different tree prints `run|restart`.

`dump` prints the first frame: `widget|`, `render|`, `focus|`, and `sem|`. `shot` writes a PPM and prints `shot|file=PATH`.

`pack` writes an Android APK or an iOS `.app`. The targets and the manual simulator launch are in `docs/ui/mobile.md`.

Records:

- `build|ok|ms=N` after a build that wrote the wasm module or the shared library.
- `build|fail|...` when that build fails. The previous output stays in place.
- `serve|url=http://127.0.0.1:PORT/` once the listener is up.
- `run|reload` after a hot reload.
- `run|restart` after a hot restart.
- `shot|file=PATH` after a shot.
- `more|N` when `--max` cut a dump.

`std` and `ui` come from `LLRST_STD` and `LLRST_UI` when those are set, otherwise from `./std` and `./ui` or the install share next to `llrc`. The page assets come from `./web` or `LLRST_WEB`.

A restart happens when a state's `layout` changes, when any `layout` is `-1` (the struct holds a closure or a function), when the widget tree shape changes, or when a wasm rebuild moves the heap base because the data segment no longer fits the reserved gap. `navigator`, `list`, and `animated_builder` return `-1` because their state holds a closure. Their `save` string is restored after the restart.
