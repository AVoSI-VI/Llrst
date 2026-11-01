# llr-ui

Builds a Llrst UI for the browser, or packs one for Android or iOS.

```
llr-ui serve FILE [--out DIR] [--port N]
llr-ui pack --target TRIPLE FILE [--out DIR] [--sign IDENT]
```

`FILE` is the program. The default out dir is `.llr-ui` and the default port is 0, which asks the kernel for a free port. The tool finds `llrc` on `PATH` and builds with `--target wasm32-unknown-unknown`. It copies `index.html`, `llr_ui.js`, and `ui/fonts/LiberationSans-Regular.ttf` into the out dir, then listens on `127.0.0.1`.

Records:

- `build|ok|ms=N` after a build that wrote `app.wasm`.
- `build|fail|...` when that build fails. The previous `app.wasm` stays in place.
- `serve|url=http://127.0.0.1:PORT/` once the listener is up.

Between accepts the server polls the source mtime and rebuilds when it changes. `std` and `ui` come from `LLRST_STD` and `LLRST_UI` when those are set, otherwise from `./std` and `./ui` or the install share next to `llrc`. The page assets come from `./web` or `LLRST_WEB`.

`pack` writes an Android APK or an iOS `.app`. The targets and the manual simulator
launch are in `docs/ui/mobile.md`. Success is `pack|apk=PATH` or `pack|app=PATH`.
A missing SDK tool is `pack|fail|...=missing`.
