//! The std prelude: every `pub` std type or trait by bare name. A repeated name is `name.dup`.
use crate::diag::{Diag, Diags, Span};
use crate::resolve::Program;
use crate::ty::DefId;

/// Records `name` for a `pub` item of the std module at `path`. A second std module declaring
/// the same `pub` type name is reported, and the first keeps the name, so every checker agrees.
pub fn add(prog: &mut Program, is_std: bool, is_pub: bool, path: &str, name: &str, id: DefId, span: Span, diags: &Diags) {
    if !is_std || !is_pub {
        return;
    }
    if prog.prelude.contains_key(name) {
        diags.push(Diag::error(span, "name.dup").kv("name", name).kv("module", path));
        return;
    }
    prog.prelude.insert(name.to_string(), id);
}
