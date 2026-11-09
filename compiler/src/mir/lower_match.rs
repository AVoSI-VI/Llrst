//! Match lowering: decision chain over arms, bindings by move or by reference.
use super::lower::Lower;
use super::*;
use crate::ast::Lit;
use crate::thir::{Arm, Expr, Pat};
use crate::ty::DefId;

struct Hit {
    local: LocalIdx,
    place: Place,
    moved: bool,
}

struct Env<'a> {
    guard: Option<&'a Expr>,
    body: &'a Expr,
    join: BlockId,
    by_ref: Option<bool>,
    /// The scrutinee place is a local, so a field move is allowed.
    scrut_local: bool,
    sp: Span,
}

#[derive(Clone)]
struct Job<'a> {
    pat: &'a Pat,
    place: Place,
}

impl<'a> Lower<'a> {
    pub fn lower_match(&mut self, scrut: &Expr, arms: &[Arm], by_ref: Option<bool>, dest: Place, sp: Span) {
        let base = match by_ref {
            Some(_) => {
                let op = self.as_operand(scrut);
                let t = self.temp(scrut.ty.clone(), scrut.span);
                self.assign(&Place::local(t), Rvalue::Use(op), scrut.span);
                Place::local(t).deref()
            }
            None => self.as_place(scrut),
        };
        let join = self.new_block();
        for arm in arms {
            let fail = self.new_block();
            let env = Env { guard: arm.guard.as_ref(), body: &arm.body, join, by_ref, scrut_local: base.is_local(), sp: arm.span };
            let jobs = vec![Job { pat: &arm.pat, place: base.clone() }];
            self.drive(jobs, fail, &mut Vec::new(), &mut Vec::new(), &env, &dest);
            self.cur = fail;
        }
        self.terminate(Term::Unreachable, sp);
        self.cur = join;
    }

    fn drive(&mut self, mut jobs: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        if jobs.is_empty() {
            self.commit(binds, drops, fail, env, dest);
            return;
        }
        let job = jobs.remove(0);
        match job.pat {
            Pat::Wild => self.drive_wild(job.place, jobs, fail, binds, drops, env, dest),
            Pat::Bind(l) => self.drive_bind(*l, job.place, jobs, fail, binds, drops, env, dest),
            Pat::Lit(l) => self.drive_lit(l, job.place, jobs, fail, binds, drops, env, dest),
            Pat::Variant { idx, subs } => self.drive_var(*idx, subs, job.place, jobs, fail, binds, drops, env, dest),
            Pat::Struct(subs) => self.drive_fields(subs, job.place, false, 0, jobs, fail, binds, drops, env, dest),
            Pat::Or(alts) => self.drive_or(alts, job.place, jobs, fail, binds, drops, env, dest),
        }
    }

    fn drive_wild(&mut self, place: Place, jobs: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        let pty = place_ty(self.prog, &self.body, &place);
        if place.proj.iter().any(|p| matches!(p, Proj::Downcast(_))) && needs_drop(self.prog, &pty) {
            drops.push(place);
        }
        self.drive(jobs, fail, binds, drops, env, dest);
    }

    fn drive_bind(&mut self, l: crate::thir::LocalId, place: Place, jobs: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        let idx = self.local_map[l.0 as usize];
        let pty = place_ty(self.prog, &self.body, &place);
        let moved = env.by_ref.is_none() && !self.is_copy(&pty);
        binds.push(Hit { local: idx, place, moved });
        self.drive(jobs, fail, binds, drops, env, dest);
    }

    fn drive_lit(&mut self, l: &Lit, place: Place, jobs: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        if let Lit::Str(s) = l {
            self.drive_str(s, place, jobs, fail, binds, drops, env, dest);
            return;
        }
        let pty = place_ty(self.prog, &self.body, &place);
        let c = super::lower::lit_const(l, &pty);
        let ok = self.new_block();
        match (l, &c) {
            (Lit::Bool(b), _) => {
                let (t, f) = if *b { (ok, fail) } else { (fail, ok) };
                self.terminate(Term::If(Operand::Copy(place), t, f), env.sp);
            }
            (_, Const::Int(v, _)) => {
                self.terminate(Term::Switch(Operand::Copy(place), vec![(*v as u128, ok)], fail), env.sp);
            }
            _ => self.terminate(Term::Goto(fail), env.sp),
        }
        self.cur = ok;
        self.drive(jobs, fail, binds, drops, env, dest);
    }

    fn drive_str(&mut self, bytes: &[u8], place: Place, jobs: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        let arg = self.borrow_str(place, env.sp);
        let lit = Operand::Const(Const::Str(bytes.to_vec()), ref_str(self.prog));
        let slot = self.temp(Ty::Bool, env.sp);
        let next = self.new_block();
        let def = str_eq(self.prog);
        self.terminate(Term::Call { func: Func::Def(def, Vec::new()), args: vec![arg, lit], dest: Place::local(slot), next, span: env.sp }, env.sp);
        self.cur = next;
        let ok = self.new_block();
        self.terminate(Term::If(Operand::Copy(Place::local(slot)), ok, fail), env.sp);
        self.cur = ok;
        self.drive(jobs, fail, binds, drops, env, dest);
    }

    fn borrow_str(&mut self, place: Place, sp: Span) -> Operand {
        let t = self.temp(ref_str(self.prog), sp);
        self.assign(&Place::local(t), Rvalue::Ref { mutable: false, place, two_phase: false }, sp);
        Operand::Copy(Place::local(t))
    }

    fn drive_var(&mut self, idx: u32, subs: &[Pat], place: Place, rest: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        let d = self.temp(Ty::Int(crate::ty::IntTy::U32), env.sp);
        self.assign(&Place::local(d), Rvalue::Discr(place.clone()), env.sp);
        let ok = self.new_block();
        self.terminate(Term::Switch(Operand::Copy(Place::local(d)), vec![(idx as u128, ok)], fail), env.sp);
        self.cur = ok;
        self.drive_fields(subs, place, true, idx, rest, fail, binds, drops, env, dest);
    }

    fn drive_fields(&mut self, subs: &[Pat], place: Place, down: bool, idx: u32, rest: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        let mut jobs = Vec::new();
        for (i, s) in subs.iter().enumerate() {
            let mut fp = place.clone();
            if down {
                fp = fp.downcast(idx);
            }
            jobs.push(Job { pat: s, place: fp.field(i as u32) });
        }
        jobs.extend(rest);
        self.drive(jobs, fail, binds, drops, env, dest);
    }

    fn drive_or(&mut self, alts: &[Pat], place: Place, rest: Vec<Job<'_>>, fail: BlockId, binds: &mut Vec<Hit>, drops: &mut Vec<Place>, env: &Env<'_>, dest: &Place) {
        let n = alts.len();
        let saved_b = binds.len();
        let saved_d = drops.len();
        for (i, alt) in alts.iter().enumerate() {
            let alt_fail = if i + 1 == n { fail } else { self.new_block() };
            let mut jobs = vec![Job { pat: alt, place: place.clone() }];
            jobs.extend(rest.iter().cloned());
            self.drive(jobs, alt_fail, binds, drops, env, dest);
            binds.truncate(saved_b);
            drops.truncate(saved_d);
            if i + 1 != n {
                self.cur = alt_fail;
            }
        }
        self.cur = fail;
    }

    fn commit(&mut self, binds: &[Hit], drops: &[Place], fail: BlockId, env: &Env<'_>, dest: &Place) {
        self.push_scope();
        let mut moved_any = false;
        for h in binds {
            if self.bind_one(h, env) {
                moved_any = true;
            }
        }
        if let Some(g) = env.guard {
            let slot = self.temp(Ty::Bool, g.span);
            self.expr_into(g, Place::local(slot), g.span);
            let gfail = self.new_block();
            let gok = self.new_block();
            self.terminate(Term::If(Operand::Copy(Place::local(slot)), gok, gfail), env.sp);
            self.cur = gfail;
            self.move_back(binds, env.sp);
            self.release_scope(env.sp);
            self.terminate(Term::Goto(fail), env.sp);
            self.cur = gok;
        }
        if env.by_ref.is_none() && moved_any {
            for d in drops {
                self.push(Stmt::Drop(d.clone()), env.sp);
            }
        }
        self.expr_into(env.body, dest.clone(), env.sp);
        self.pop_scope(env.sp);
        self.terminate(Term::Goto(env.join), env.sp);
    }

    fn bind_one(&mut self, h: &Hit, env: &Env<'_>) -> bool {
        let rv = match env.by_ref {
            Some(m) => {
                self.scopes.last_mut().unwrap().all.push(h.local);
                Rvalue::Ref { mutable: m, place: h.place.clone(), two_phase: false }
            }
            None => {
                let pty = place_ty(self.prog, &self.body, &h.place);
                let sc = self.scopes.last_mut().unwrap();
                sc.all.push(h.local);
                if needs_drop(self.prog, &pty) {
                    sc.drops.push(h.local);
                }
                let op = if self.is_copy(&pty) {
                    Operand::Copy(h.place.clone())
                } else {
                    if !env.scrut_local {
                        let code = if h.place.has_deref() { "move.deref" } else { "move.field" };
                        self.diags.push(crate::diag::Diag::error(env.sp, code).kv("fix", "match on (& ...)"));
                    }
                    Operand::Move(h.place.clone())
                };
                Rvalue::Use(op)
            }
        };
        let moved = matches!(rv, Rvalue::Use(Operand::Move(_)));
        self.assign(&Place::local(h.local), rv, env.sp);
        moved
    }

    fn move_back(&mut self, binds: &[Hit], sp: Span) {
        for h in binds {
            if h.moved {
                self.assign(&h.place.clone(), Rvalue::Use(Operand::Move(Place::local(h.local))), sp);
            }
        }
    }

    fn release_scope(&mut self, sp: Span) {
        let s = self.scopes.last().unwrap();
        let drops = s.drops.clone();
        let all = s.all.clone();
        for l in drops.iter().rev() {
            self.push(Stmt::Drop(Place::local(*l)), sp);
        }
        for l in &all {
            self.push(Stmt::StorageDead(*l), sp);
        }
    }
}

fn ref_str(prog: &crate::resolve::Program) -> Ty {
    let id = prog.prelude_type("str").expect("str");
    Ty::Ref { mutable: false, lt: None, inner: Box::new(Ty::Adt(id, Vec::new())) }
}

fn str_eq(prog: &crate::resolve::Program) -> DefId {
    let owner = prog.prelude_type("str").expect("str");
    *prog.methods.get(&owner).and_then(|m| m.get("eq")).expect("str.eq")
}
