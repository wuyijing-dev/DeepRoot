//! SMP bring-up — SBI HSM secondary harts + online mask (1.7).

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::mm::sv39;
use crate::println;
use crate::sbi;
use crate::sched;
use crate::timer;
use crate::trap;

/// Teaching board: QEMU virt with `-smp 2` (DTS lists both CPUs).
pub const MAX_HARTS: usize = 4;
pub const HART_STACK_SIZE: usize = 64 * 1024;

unsafe extern "C" {
    static __deeproot_hart_stacks: u8;
}

static BOOT_HART: AtomicUsize = AtomicUsize::new(0);
static ONLINE_MASK: AtomicUsize = AtomicUsize::new(0);
static HART_COUNT: AtomicUsize = AtomicUsize::new(1);
static MM_READY: AtomicBool = AtomicBool::new(false);
static SCHED_READY: AtomicBool = AtomicBool::new(false);

/*
 * set_tp - park hart id in `tp` (used by trap / sched / timer)
 */
#[inline]
pub fn set_tp(hartid: usize) {
    unsafe {
        core::arch::asm!("mv tp, {}", in(reg) hartid, options(nomem, nostack));
    }
}

#[inline]
pub fn hart_id() -> usize {
    let mut tp: usize;
    unsafe {
        core::arch::asm!("mv {}, tp", out(reg) tp, options(nostack, nomem));
    }
    tp
}

pub fn boot_hart() -> usize {
    BOOT_HART.load(Ordering::Relaxed)
}

pub fn hart_count() -> usize {
    HART_COUNT.load(Ordering::Relaxed)
}

pub fn is_online(hart: usize) -> bool {
    (ONLINE_MASK.load(Ordering::Acquire) & (1 << hart)) != 0
}

pub fn online_mask() -> usize {
    ONLINE_MASK.load(Ordering::Acquire)
}

fn mark_online(hart: usize) {
    ONLINE_MASK.fetch_or(1 << hart, Ordering::Release);
}

pub fn mark_mm_ready() {
    MM_READY.store(true, Ordering::Release);
}

pub fn mark_sched_ready() {
    SCHED_READY.store(true, Ordering::Release);
}

pub fn sched_ready() -> bool {
    SCHED_READY.load(Ordering::Acquire)
}

/*
 * stack_top - kernel stack top for @hart (grows down)
 */
pub fn stack_top(hart: usize) -> usize {
    debug_assert!(hart < MAX_HARTS);
    let base = core::ptr::addr_of!(__deeproot_hart_stacks) as usize;
    base + (hart + 1) * HART_STACK_SIZE
}

/*
 * init_boot_hart - primary path after OpenSBI; set tp and online bit 0
 */
pub fn init_boot_hart(hartid: usize) {
    BOOT_HART.store(hartid, Ordering::Relaxed);
    set_tp(hartid);
    mark_online(hartid);
    HART_COUNT.store(1, Ordering::Relaxed);
}

/*
 * boot_secondaries - HSM-start harts 1..n-1 after Sv39 is live
 *
 * @want: desired count from platform (DTS / `-smp`); capped at MAX_HARTS.
 */
pub fn boot_secondaries(want: usize) {
    let want = want.clamp(1, MAX_HARTS);
    let entry = secondary_entry_pa();
    let boot = boot_hart();
    let mut online = 1usize;

    for h in 0..want {
        if h == boot {
            continue;
        }
        match sbi::hart_start(h, entry, 0) {
            Ok(()) => {
                let mut spins = 0usize;
                while !is_online(h) && spins < 5_000_000 {
                    core::hint::spin_loop();
                    spins += 1;
                }
                if is_online(h) {
                    online += 1;
                    println!("smp: hart {} online (HSM start ok)", h);
                } else {
                    println!("smp: hart {} start timed out", h);
                }
            }
            Err(e) => {
                println!("smp: hart_start({}) failed err={}", h, e);
            }
        }
    }

    HART_COUNT.store(online, Ordering::Release);
    println!(
        "smp: {} hart(s) online mask={:#x} (boot={})",
        online,
        online_mask(),
        boot
    );
}

fn secondary_entry_pa() -> usize {
    unsafe extern "C" {
        fn _secondary_start();
    }
    _secondary_start as *const () as usize
}

/*
 * secondary_main - entry from `_secondary_start` (a0 already in tp)
 */
#[no_mangle]
pub extern "C" fn secondary_main(hartid: usize) -> ! {
    set_tp(hartid);

    while !MM_READY.load(Ordering::Acquire) {
        core::hint::spin_loop();
    }

    let root = sv39::kernel_root_pa();
    sv39::activate(root);

    trap::init_secondary();
    timer::init(hartid);
    sbi::enable_supervisor_soft_irq();

    mark_online(hartid);
    println!("smp: secondary hart={} ready", hartid);

    while !SCHED_READY.load(Ordering::Acquire) {
        core::hint::spin_loop();
    }

    if let Some(idle) = sched::idle_id_for(hartid) {
        sched::enter_first(idle);
    }
    loop {
        sbi::hart_suspend_idle();
    }
}

/*
 * ipi_wake - nudge @hart out of WFI (SSIP via SBI IPI)
 */
pub fn ipi_wake(hart: usize) {
    if hart >= MAX_HARTS || hart == hart_id() || !is_online(hart) {
        return;
    }
    let _ = sbi::send_ipi_hart(hart);
}

/*
 * ipi_wake_others - wake every online hart except self
 */
pub fn ipi_wake_others() {
    let me = hart_id();
    let mask = online_mask();
    for h in 0..MAX_HARTS {
        if h != me && (mask & (1 << h)) != 0 {
            let _ = sbi::send_ipi_hart(h);
        }
    }
}
