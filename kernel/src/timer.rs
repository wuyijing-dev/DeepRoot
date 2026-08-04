//! Supervisor timer via SBI TIME (0.6.2).
//!
//! QEMU virt ACLINT MTIMER is typically 10 MHz (see OpenSBI boot log).

use core::sync::atomic::{AtomicU64, Ordering};

use crate::println;

const SBI_EXT_TIME: usize = 0x54494D45; /* "TIME" */
const SBI_TIME_SET_TIMER: usize = 0;

/// Supervisor timer interrupt pending / enable bit in `sip`/`sie`.
pub const SIE_STIE: usize = 1 << 5;
/// Global interrupt enable in `sstatus`.
pub const SSTATUS_SIE: usize = 1 << 1;

/// Default quantum ≈ 10 ms at 10 MHz.
pub const TICKS_PER_SLICE: u64 = 100_000;

static TICKS: AtomicU64 = AtomicU64::new(0);
static HART_ID: AtomicU64 = AtomicU64::new(0);

/*
 * time_now - read the `time` CSR (shadow of mtime)
 */
#[inline]
pub fn time_now() -> u64 {
    let mut v: u64;
    unsafe {
        core::arch::asm!("csrr {}, time", out(reg) v, options(nostack, nomem));
    }
    v
}

fn sbi_set_timer(abs: u64) {
    let mut error: isize;
    let mut value: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") SBI_EXT_TIME,
            in("a6") SBI_TIME_SET_TIMER,
            inout("a0") abs as usize => error,
            inout("a1") 0usize => value,
            in("a2") 0usize,
            options(nostack),
        );
    }
    let _ = (error, value);
}

/*
 * init - record boot hart and arm the first quantum
 * @hartid: OpenSBI a0 (multi-hart prep: one timer per hart later)
 */
pub fn init(hartid: usize) {
    HART_ID.store(hartid as u64, Ordering::Relaxed);
    /* Enable supervisor timer interrupts in sie. */
    unsafe {
        core::arch::asm!(
            "csrs sie, {}",
            in(reg) SIE_STIE,
            options(nomem, nostack),
        );
    }
    arm_next();
    println!(
        "timer: hart={} slice={} cycles (SBI TIME)",
        hartid, TICKS_PER_SLICE
    );
}

pub fn hart_id() -> usize {
    HART_ID.load(Ordering::Relaxed) as usize
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

/*
 * arm_next - program the next absolute timer deadline
 */
pub fn arm_next() {
    let next = time_now().wrapping_add(TICKS_PER_SLICE);
    sbi_set_timer(next);
}

/*
 * on_interrupt - clear/rearm timer; returns tick count
 */
pub fn on_interrupt() -> u64 {
    /* Clear STIP by setting timer far ahead then re-arming, per SBI. */
    sbi_set_timer(u64::MAX);
    let t = TICKS.fetch_add(1, Ordering::Relaxed) + 1;
    arm_next();
    t
}

/*
 * enable_s_ie - allow S-mode to take interrupts when SPIE→SIE on sret
 *
 * We do not set sstatus.SIE while running in the kernel trap path; U-mode
 * resumes with SPIE so the next U-mode timeslice can be preempted.
 */
pub fn note_preempt_ready() {
    println!("timer: preemption armed (STIE); ticks so far={}", ticks());
}
