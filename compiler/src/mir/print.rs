//! Textual MIR dump for `llrc emit-mir`.
use super::*;
use std::fmt::Write;

pub fn print_body(b: &Body) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "fn {} ({} args)", b.name, b.arg_count);
    for (i, l) in b.locals.iter().enumerate() {
        let _ = writeln!(s, "  _{}: {} ; {}{}", i, l.ty, l.name, if l.mutable { " mut" } else { "" });
    }
    for (bi, blk) in b.blocks.iter().enumerate() {
        let _ = writeln!(s, " bb{}:", bi);
        for (st, _) in &blk.stmts {
            let _ = writeln!(s, "   {}", stmt(st));
        }
        let _ = writeln!(s, "   {}", term(&blk.term));
    }
    s
}

fn place(p: &Place) -> String {
    let mut s = format!("_{}", p.local);
    for pr in &p.proj {
        match pr {
            Proj::Field(i) => s = format!("{}.{}", s, i),
            Proj::Deref => s = format!("(*{})", s),
            Proj::Downcast(v) => s = format!("({} as v{})", s, v),
        }
    }
    s
}

fn operand(o: &Operand) -> String {
    match o {
        Operand::Copy(p) => format!("copy {}", place(p)),
        Operand::Move(p) => format!("move {}", place(p)),
        Operand::Const(c, _) => match c {
            Const::Int(v, t) => format!("{}{}", v, t.name()),
            Const::Float(v, _) => format!("{}", v),
            Const::Bool(b) => format!("{}", b),
            Const::Str(s) => format!("{:?}", String::from_utf8_lossy(s)),
            Const::Unit => "()".into(),
            Const::Fn(d, _) => format!("fn#{}", d.0),
        },
    }
}

fn ops(v: &[Operand]) -> String {
    v.iter().map(operand).collect::<Vec<_>>().join(", ")
}

fn rvalue(r: &Rvalue) -> String {
    match r {
        Rvalue::Use(o) => operand(o),
        Rvalue::Ref { mutable, place: p, two_phase } => format!("&{}{} {}", if *mutable { "mut " } else { "" }, if *two_phase { "2p" } else { "" }, place(p)),
        Rvalue::Bin(op, a, b) => format!("{:?}({}, {})", op, operand(a), operand(b)),
        Rvalue::Neg(a) => format!("neg {}", operand(a)),
        Rvalue::WNeg(a) => format!("wneg {}", operand(a)),
        Rvalue::Not(a) => format!("not {}", operand(a)),
        Rvalue::Cast(a, t) => format!("{} as {}", operand(a), t),
        Rvalue::Struct(t, o) => format!("{} {{{}}}", t, ops(o)),
        Rvalue::Variant(t, i, o) => format!("{}::v{} {{{}}}", t, i, ops(o)),
        Rvalue::Discr(p) => format!("discr {}", place(p)),
        Rvalue::PtrNull => "null".into(),
        Rvalue::PtrAdd(a, b, _) => format!("ptradd({}, {})", operand(a), operand(b)),
        Rvalue::PtrEq(a, b) => format!("ptreq({}, {})", operand(a), operand(b)),
        Rvalue::Argc => "argc".into(),
        Rvalue::Argv => "argv".into(),
        Rvalue::Sizeof(t) => format!("sizeof {}", t),
        Rvalue::Vtable(..) => "vtable".into(),
        Rvalue::Static(_) => "static".into(),
        Rvalue::Closure { idx, caps } => format!("clos{} {{{}}}", idx, ops(caps)),
    }
}

fn stmt(s: &Stmt) -> String {
    match s {
        Stmt::Assign(p, r) => format!("{} = {}", place(p), rvalue(r)),
        Stmt::Drop(p) => format!("drop {}", place(p)),
        Stmt::Forget(l) => format!("forget _{}", l),
        Stmt::StorageDead(l) => format!("dead _{}", l),
        Stmt::Nop => "nop".into(),
        Stmt::InlineLoans { dest, .. } => format!("loans _{}", dest),
        Stmt::TracePush { name, .. } => format!("trace.push {}", name),
        Stmt::TracePop { .. } => "trace.pop".into(),
    }
}

fn term(t: &Term) -> String {
    match t {
        Term::Goto(b) => format!("goto bb{}", b),
        Term::If(o, a, b) => format!("if {} bb{} bb{}", operand(o), a, b),
        Term::Switch(o, cases, d) => {
            let c: Vec<String> = cases.iter().map(|(v, b)| format!("{}->bb{}", v, b)).collect();
            format!("switch {} [{}] else bb{}", operand(o), c.join(" "), d)
        }
        Term::Call { func, args, dest, next, .. } => {
            let f = match func {
                Func::Def(d, _) => format!("fn#{}", d.0),
                Func::Ptr(o) | Func::Clos(o) => operand(o),
                Func::Dyn { slot, .. } => format!("dyn#{slot}"),
            };
            format!("{} = {}({}) -> bb{}", place(dest), f, ops(args), next)
        }
        Term::Return => "return".into(),
        Term::Unreachable => "unreachable".into(),
        Term::None => "<none>".into(),
    }
}
