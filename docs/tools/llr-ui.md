# llr-ui

Builds a Llrst UI for the browser and serves the canvas page.

```
llr-ui serve FILE [--out DIR] [--port N]
```

`FILE` is the program. The default out dir is `.llr-ui` and the default port is 0, which asks the kernel for a free port. The tool finds `llrc` on `PATH` and builds with `--target wasm32-unknown-unknown`. It copies `index.html`, `llr_ui.js`, and `ui/fonts/LiberationSans-Regular.ttf` into the out dir, then listens on `127.0.0.1`.

Records:

- `build|ok|ms=N` after a build that wrote `app.wasm`.
- `build|fail|...` when that build fails. The previous `app.wasm` stays in place.
- `serve|url=http://127.0.0.1:PORT/` once the listener is up.

Between accepts the server polls the source mtime and rebuilds when it changes. `std` and `ui` come from `LLRST_STD` and `LLRST_UI` when those are set, otherwise from `./std` and `./ui` or the install share next to `llrc`. The page assets come from `./web` or `LLRST_WEB`.
