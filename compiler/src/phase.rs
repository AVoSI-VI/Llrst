//! Per-phase timers. Set `LLRST_TIME` to print `time|<phase>|<ms>` on stderr.
//! Unset, nothing is printed and the hot path does not take a clock sample.
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

pub const LOAD: usize = 0;
pub const PARSE: usize = 1;
pub const COLLECT: usize = 2;
pub const RESOLVE: usize = 3;
pub const CHECK: usize = 4;
pub const BORROW: usize = 5;
pub const LOWER: usize = 6;
pub const CODEGEN: usize = 7;
pub const LINK: usize = 8;

const NAMES: [&str; 9] = [
    "load", "parse", "collect", "resolve", "check", "borrow", "lower", "codegen", "link",
];

// Charged only while collect runs, so check's resolve_ty calls stay in check.
static RESOLVE_ON: AtomicBool = AtomicBool::new(false);
static RESOLVE_NS: AtomicU64 = AtomicU64::new(0);

pub struct Phases {
    on: bool,
    ns: RefCell<[u128; 9]>,
}

impl Phases {
    pub fn from_env() -> Self {
        Phases { on: std::env::var_os("LLRST_TIME").is_some(), ns: RefCell::new([0; 9]) }
    }

    pub fn guard(&self, slot: usize) -> Guard<'_> {
        Guard { phases: self, t0: self.tick(), kind: Kind::Add(slot) }
    }

    /// Whole load, minus parse, collect, and resolve recorded inside it.
    pub fn guard_load(&self) -> Guard<'_> {
        let ns = *self.ns.borrow();
        Guard {
            phases: self,
            t0: self.tick(),
            kind: Kind::LoadNet { parse: ns[PARSE], collect: ns[COLLECT], resolve: ns[RESOLVE] },
        }
    }

    /// collect(), with resolve_ty time moved into the resolve bucket.
    pub fn guard_collect(&self) -> Guard<'_> {
        if self.on {
            RESOLVE_NS.store(0, Ordering::Relaxed);
            RESOLVE_ON.store(true, Ordering::Relaxed);
        }
        Guard { phases: self, t0: self.tick(), kind: Kind::CollectNet }
    }

    pub fn report(&self) {
        if !self.on {
            return;
        }
        let ns = self.ns.borrow();
        for (i, name) in NAMES.iter().enumerate() {
            eprintln!("time|{name}|{}", ns[i] / 1_000_000);
        }
    }

    fn tick(&self) -> Option<Instant> {
        self.on.then(Instant::now)
    }

    fn add(&self, slot: usize, dt: u128) {
        let mut ns = self.ns.borrow_mut();
        ns[slot] = ns[slot].saturating_add(dt);
    }
}

enum Kind {
    Add(usize),
    LoadNet { parse: u128, collect: u128, resolve: u128 },
    CollectNet,
}

pub struct Guard<'a> {
    phases: &'a Phases,
    t0: Option<Instant>,
    kind: Kind,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        let Some(t0) = self.t0 else { return };
        let wall = t0.elapsed().as_nanos();
        match self.kind {
            Kind::Add(slot) => self.phases.add(slot, wall),
            Kind::CollectNet => {
                RESOLVE_ON.store(false, Ordering::Relaxed);
                let res = RESOLVE_NS.swap(0, Ordering::Relaxed) as u128;
                self.phases.add(RESOLVE, res);
                self.phases.add(COLLECT, wall.saturating_sub(res));
            }
            Kind::LoadNet { parse, collect, resolve } => {
                self.phases.add(LOAD, wall);
                let ns = self.phases.ns.borrow();
                let sub = ns[PARSE].saturating_sub(parse)
                    + ns[COLLECT].saturating_sub(collect)
                    + ns[RESOLVE].saturating_sub(resolve);
                drop(ns);
                let mut ns = self.phases.ns.borrow_mut();
                ns[LOAD] = ns[LOAD].saturating_sub(sub);
            }
        }
    }
}

pub fn resolve_on() -> bool {
    RESOLVE_ON.load(Ordering::Relaxed)
}

pub fn resolve_pause() {
    RESOLVE_ON.store(false, Ordering::Relaxed);
}

pub fn resolve_resume(ns: u128) {
    RESOLVE_NS.fetch_add(ns.min(u128::from(u64::MAX)) as u64, Ordering::Relaxed);
    RESOLVE_ON.store(true, Ordering::Relaxed);
}
