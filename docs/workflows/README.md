# Agent workflows

Numbered command sequences for the common jobs. Each step names the tool, the record you
read back, and the decision it drives. Read `docs/canon/README.md` first.

| file | job |
|------|-----|
| [explore.md](explore.md) | learn a codebase: find, grep, read |
| [edit.md](edit.md) | make a change: read, edit, fmt, test |
| [debug.md](debug.md) | fix a compile or test failure from `E|` / `F|` lines |
| [refactor.md](refactor.md) | rename, extract, move between modules |
| [new-tool.md](new-tool.md) | add a tool to `tools/` and install it |
| [std.md](std.md) | grow the standard library |

Record parsing rules shared by every workflow:

1. Split on `|`; the first field is the tag (`E W P F ok fail diff more done stat usage file status log line hit dir path`).
2. `E|FILE|LINE|COL|LEN|CODE|k=v...`: open FILE at LINE:COL; CODE names the rule; `fix=`
   is a drop-in suggestion.
3. `more|...` means the list was truncated: narrow the query or raise `--max`.
4. Exit 2 is a negative answer, not a tool failure.
