//! Root Ledger — DeepRoot innovation #1: a teaching microscope for causality.
//!
//! Fixed ring of `LedgerEvent` records. Not durable, not secure — printable.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::println;
use deeproot_abi::{LedgerEvent, LedgerKind};

/// Power-of-two capacity keeps indexing cheap for learners to reason about.
pub const LEDGER_CAP: usize = 64;

pub struct RootLedger {
    events: UnsafeCell<[LedgerEvent; LEDGER_CAP]>,
    head: AtomicUsize,
    count: AtomicUsize,
}

/*
 * Safety: Boot Seed is single-hart. Atomically bumping head is preparation
 * for 0.6.x multi-hart; still no IRQ nesting around record() yet.
 */
unsafe impl Sync for RootLedger {}

impl RootLedger {
    pub const fn new() -> Self {
        Self {
            events: UnsafeCell::new(
                [LedgerEvent {
                    kind: 0,
                    _pad: [0; 3],
                    a0: 0,
                    a1: 0,
                    a2: 0,
                }; LEDGER_CAP],
            ),
            head: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
        }
    }

    /*
     * record - append one event to the ring
     * @kind: LedgerKind discriminant
     * @a0..a2: event-specific payload (documented per call site)
     *
     * Overwrites the oldest entry when full — learners should dump early.
     */
    pub fn record(&self, kind: LedgerKind, a0: u32, a1: u32, a2: u32) {
        let idx = self.head.fetch_add(1, Ordering::Relaxed) % LEDGER_CAP;
        unsafe {
            (*self.events.get())[idx] = LedgerEvent::new(kind, a0, a1, a2);
        }
        let c = self.count.load(Ordering::Relaxed);
        if c < LEDGER_CAP {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /*
     * dump_to_console - print the ledger oldest→newest for worksheets
     *
     * Ordering: we reconstruct using head and count. Good enough for Boot Seed.
     */
    pub fn dump_to_console(&self) {
        let count = self.count.load(Ordering::Relaxed).min(LEDGER_CAP);
        let head = self.head.load(Ordering::Relaxed);
        let start = if count < LEDGER_CAP {
            0
        } else {
            head % LEDGER_CAP
        };

        let events = unsafe { &*self.events.get() };
        println!("---- Root Ledger ({} events) ----", count);
        for i in 0..count {
            let idx = (start + i) % LEDGER_CAP;
            let e = events[idx];
            println!(
                "  [{:>2}] kind={} a0={} a1={} a2={}",
                i, e.kind, e.a0, e.a1, e.a2
            );
        }
        println!("---- end ledger ----");
    }
}

pub static LEDGER: RootLedger = RootLedger::new();

pub fn init() {
    /* Placeholder for future per-CPU ledgers; keeps call sites stable. */
}
