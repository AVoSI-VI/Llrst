# llr-lsp

Language server over stdio. Speaks LSP framing: `Content-Length: N` then `\r\n\r\n` then N bytes of JSON-RPC.

```
llr-lsp
```

No arguments. `--help` prints `usage|llr-lsp|...` and exits 0. The process runs until stdin closes, or until the `exit` notification.

`initialize` answers `textDocumentSync` 1 (full documents) and `definitionProvider` true. `shutdown` answers `null`. `exit` after `shutdown` ends with code 0; `exit` before it ends with code 1.

Diagnostics: `textDocument/didOpen`, `didChange`, and `didSave` write the buffer beside the file as `.llr-lsp-<pid>.llr` and run `llrc check` on it. `$LLRC` is split on spaces, so `llrc --std ./std` works. Each `E|` line is severity 1 and each `W|` line is severity 2. The message is `CODE|k=v...`. Line and column are converted from 1-based to LSP's 0-based range. The result is `textDocument/publishDiagnostics`. A missing compiler is one diagnostic `lsp.llrc`.

Definition: `textDocument/definition` reads the word at the cursor (`[A-Za-z0-9_]`) and runs `llr-grep` in the workspace root (`rootUri`, else the first `workspaceFolders` entry, else `rootPath`, else the file's directory). The pattern is the resolver's def heads:

```
\((fn|extern|struct|enum|const|trait) NAME\b
```

`llr-grep` is the copy beside `llr-lsp` when that file exists, otherwise `llr-grep` on `PATH`. A hit in the same file wins. The result is one `Location`, or `null`.

Exit: 0 on EOF or `exit` after `shutdown`, 1 on `exit` without `shutdown`.
