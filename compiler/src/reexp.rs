//! `(pub (use ...))` shares one name set with the module's own items.
use crate::diag::{Diag, Diags};
use crate::resolve::Program;
use crate::ty::DefId;
use std::collections::{HashMap, HashSet};

pub fn check(prog: &Program, diags: &Diags) {
    for m in &prog.modules {
        let mut claimed: HashMap<String, DefId> = m.items.iter().map(|(n, id)| (n.clone(), *id)).collect();
        for (target, span) in &m.reexports {
            let mut names = pub_exports(prog, *target, &mut HashSet::new());
            names.sort_by(|a, b| a.0.cmp(&b.0));
            for (name, id) in names {
                if let Some(old) = claimed.get(&name).copied() {
                    if old != id {
                        diags.push(Diag::error(*span, "name.dup").kv("name", name));
                    }
                } else {
                    claimed.insert(name, id);
                }
            }
        }
    }
}

/// Public names visible through `module`, skipping ones shadowed by a local item.
fn pub_exports(prog: &Program, module: u32, seen: &mut HashSet<u32>) -> Vec<(String, DefId)> {
    if !seen.insert(module) {
        return Vec::new();
    }
    let m = &prog.modules[module as usize];
    let mut locals: Vec<(String, DefId)> = m.items.iter().map(|(n, id)| (n.clone(), *id)).collect();
    locals.sort_by(|a, b| a.0.cmp(&b.0));
    let blocked: HashSet<String> = locals.iter().map(|(n, _)| n.clone()).collect();
    let mut out = Vec::new();
    for (name, id) in locals {
        if prog.def(id).is_pub {
            out.push((name, id));
        }
    }
    let targets: Vec<u32> = m.reexports.iter().map(|(t, _)| *t).collect();
    for mi in targets {
        for (name, id) in pub_exports(prog, mi, seen) {
            if blocked.contains(&name) || out.iter().any(|(n, _)| n == &name) {
                continue;
            }
            out.push((name, id));
        }
    }
    out
}
