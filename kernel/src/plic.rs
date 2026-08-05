//! PLIC (Platform-Level Interrupt Controller) for QEMU virt — 1.16.
//!
//! S-mode context for hart H is `2*H + 1` (M-mode is even). Teaching path:
//! enable virtio IRQs, claim/complete on supervisor external interrupt.

use core::sync::atomic::{AtomicU64, Ordering};

use crate::mm::sv39;
use crate::println;

const PLIC_BASE: usize = 0x0c00_0000;
const PLIC_SIZE: usize = 0x0400_0000;
const PLIC_PRIORITY: usize = PLIC_BASE;
const PLIC_ENABLE: usize = PLIC_BASE + 0x2000;
const PLIC_THRESHOLD: usize = PLIC_BASE + 0x200_000;
const PLIC_CLAIM: usize = PLIC_BASE + 0x200_004;
const CONTEXT_STRIDE: usize = 0x1000;
const ENABLE_STRIDE: usize = 0x80;
const MAX_IRQ: usize = 64;

/// Latched IRQ bits so waiters do not miss a pulse before SYS_IRQ_WAIT.
static PENDING: AtomicU64 = AtomicU64::new(0);

#[inline]
fn s_context(hart: usize) -> usize {
    hart * 2 + 1
}

fn w32(addr: usize, v: u32) {
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, v);
    }
}

fn r32(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

/*
 * init - map PLIC, set priorities, enable IRQs 1..8 for each hart S-context
 */
pub fn init(hart_count: usize) {
    sv39::map_mmio_range(PLIC_BASE, PLIC_SIZE);
    for irq in 1..=8u32 {
        w32(PLIC_PRIORITY + (irq as usize) * 4, 1);
    }
    let n = hart_count.max(1).min(8);
    for h in 0..n {
        let ctx = s_context(h);
        w32(PLIC_THRESHOLD + ctx * CONTEXT_STRIDE, 0);
        /* Enable bits 1..8 in the first enable word. */
        let en = PLIC_ENABLE + ctx * ENABLE_STRIDE;
        let cur = r32(en);
        w32(en, cur | 0x1fe); /* bits 1..8 */
    }
    println!("plic: ready (S-mode ctx, irq 1..=8)");
}

pub fn latch(irq: u32) {
    if irq == 0 || irq as usize >= MAX_IRQ {
        return;
    }
    PENDING.fetch_or(1u64 << irq, Ordering::SeqCst);
}

pub fn take(irq: u32) -> bool {
    if irq == 0 || irq as usize >= MAX_IRQ {
        return false;
    }
    let bit = 1u64 << irq;
    let prev = PENDING.fetch_and(!bit, Ordering::SeqCst);
    (prev & bit) != 0
}

/*
 * claim - read claim register for this hart's S-context (0 = none)
 */
pub fn claim(hart: usize) -> u32 {
    let ctx = s_context(hart);
    r32(PLIC_CLAIM + ctx * CONTEXT_STRIDE)
}

/*
 * complete - write claim id back to complete the IRQ
 */
pub fn complete(hart: usize, irq: u32) {
    if irq == 0 {
        return;
    }
    let ctx = s_context(hart);
    w32(PLIC_CLAIM + ctx * CONTEXT_STRIDE, irq);
}
