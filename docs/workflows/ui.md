# Workflow: a widget

Read `docs/canon/ui.md` first. One widget per task. The tree is ordinary constructor calls.

1. Add the widget in `ui/` or in the program. A `state` implements `save`, `load`, and
   `layout`. `layout` is `(as i64 (sizeof TheStruct))`, or `-1` when the struct holds a
   closure or a function pointer.
2. Test it headless. `llrc build tests/ui/NAME.llr --std std --ui ui -o /tmp/ui` then run
   it. Stdout is `widget|`, `render|`, `focus|`, and `sem|` lines from `app.dump`. A
   `.ppm` golden compares `--shot`. No window has to open.
3. Run it. `llr-ui run FILE` builds a shared library and keeps the element and render
   arenas in the launcher. A source change with the same `layout` values prints
   `run|reload` and re-runs `build`. A layout change, a closure in state, or a different
   widget tree prints `run|restart`: the tool writes `save`, drops the old library, and
   `load`s into a fresh tree. `llr-ui serve FILE` does the same for the browser page:
   the heap is copied when the wasm heap base is unchanged and `llr_keep` is 1, otherwise
   the page restores the blob from `sessionStorage`. `llr-ui pack --target TRIPLE FILE`
   is the mobile package from `docs/ui/mobile.md`.
4. Fix a layout bug from the dump. `llr-ui dump FILE --w 80 --h 48` prints the first
   frame. `widget|id|name|key` is the element tree, `render|id|x|y|w|h` is the box,
   `sem|id|role|label|x|y|w|h` is semantics. `--max N` cuts the list and ends with
   `more|N`. `llr-ui shot FILE OUT.ppm --w 400 --h 300` writes the pixels and prints
   `shot|file=OUT.ppm`.
