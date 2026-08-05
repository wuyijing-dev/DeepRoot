//! Loadable module / service registry (1.10–1.13).
//!
//! Boot canopy + path-spawned servers are recorded by **name** so clients can
//! [`lookup_badge`] a badge and mint a fresh Endpoint into their CapSpace (1.13).
//! Not D-Bus / not Linux kmod.

use core::cell::UnsafeCell;

use crate::println;
use crate::sync::SpinLock;

pub const MAX_MODULES: usize = 16;
pub const NAME_MAX: usize = 24;

pub struct Entry {
    pub used: bool,
    pub name_len: usize,
    pub name: [u8; NAME_MAX],
    pub badge: u64,
    pub sched_id: usize,
    pub cap_slot: usize,
}

impl Entry {
    const fn empty() -> Self {
        Self {
            used: false,
            name_len: 0,
            name: [0; NAME_MAX],
            badge: 0,
            sched_id: 0,
            cap_slot: 0,
        }
    }

    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

struct Table {
    entries: [Entry; MAX_MODULES],
}

struct Cell(UnsafeCell<Table>);
unsafe impl Sync for Cell {}

static LOCK: SpinLock = SpinLock::new();
static TABLE: Cell = Cell(UnsafeCell::new(Table {
    entries: [const { Entry::empty() }; MAX_MODULES],
}));

fn table() -> &'static mut Table {
    unsafe { &mut *TABLE.0.get() }
}

fn find_name(t: &Table, name: &str) -> Option<usize> {
    t.entries.iter().position(|e| e.used && e.name_str() == name)
}

fn find_badge(t: &Table, badge: u64) -> Option<usize> {
    t.entries.iter().position(|e| e.used && e.badge == badge)
}

/*
 * register - record a named service (unique name + unique badge)
 */
pub fn register(name: &str, badge: u64, sched_id: usize, cap_slot: usize) -> bool {
    if name.is_empty() || badge == 0 {
        return false;
    }
    let _g = LOCK.lock();
    let t = table();
    if find_name(t, name).is_some() || find_badge(t, badge).is_some() {
        return false;
    }
    let Some(idx) = t.entries.iter().position(|e| !e.used) else {
        return false;
    };
    let n = name.len().min(NAME_MAX);
    let e = &mut t.entries[idx];
    *e = Entry::empty();
    e.used = true;
    e.name_len = n;
    e.name[..n].copy_from_slice(name.as_bytes());
    e.badge = badge;
    e.sched_id = sched_id;
    e.cap_slot = cap_slot;
    println!(
        "module: loaded '{}' badge={:#x} sched={} slot={}",
        e.name_str(),
        badge,
        sched_id,
        cap_slot
    );
    true
}

/*
 * lookup_badge - resolve service name → IPC badge
 */
pub fn lookup_badge(name: &str) -> Option<u64> {
    let _g = LOCK.lock();
    let t = table();
    find_name(t, name).map(|i| t.entries[i].badge)
}

/*
 * list - dump registered modules to the console
 */
pub fn list() {
    let _g = LOCK.lock();
    let t = table();
    let mut n = 0usize;
    println!("module: registry");
    for e in t.entries.iter() {
        if e.used {
            println!(
                "  {} badge={:#x} sched={} slot={}",
                e.name_str(),
                e.badge,
                e.sched_id,
                e.cap_slot
            );
            n += 1;
        }
    }
    if n == 0 {
        println!("  (empty — try: modload moddemo)");
    }
}
