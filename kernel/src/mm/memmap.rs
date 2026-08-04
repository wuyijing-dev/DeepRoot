//! Physical memory map discovery (0.2 / 1.5).
//!
//! Prefers FDT `/memory` via [`crate::fdt`]. Falls back to QEMU virt defaults.

use super::layout::{
    align_down, ekernel, PhysAddr, DRAM_END_DEFAULT, DRAM_START, PAGE_SIZE,
};
use crate::fdt;

#[allow(dead_code)]
pub struct MemoryMap {
    pub ram_start: PhysAddr,
    pub ram_end: PhysAddr,
    pub free_start: PhysAddr,
    pub free_end: PhysAddr,
    pub dtb_pa: PhysAddr,
}

/*
 * discover - build a MemoryMap from FDT or fallback constants
 * @dtb_pa: physical address of FDT blob (OpenSBI a1); 0 means unknown
 *
 * Caller should run `fdt::probe(dtb_pa)` first so memory_reg() is populated.
 */
pub fn discover(dtb_pa: usize) -> MemoryMap {
    let (ram_start, ram_end) = fdt::memory_reg()
        .filter(|(s, e)| sane_dram(*s, *e))
        .unwrap_or((PhysAddr::new(DRAM_START), PhysAddr::new(DRAM_END_DEFAULT)));

    let mut free_start = ekernel().align_up(PAGE_SIZE);
    if free_start < ram_start {
        free_start = ram_start;
    }

    let mut free_end = ram_end.align_down(PAGE_SIZE);

    if dtb_pa != 0 {
        let dtb = PhysAddr::new(dtb_pa);
        if dtb >= free_start && dtb < free_end {
            free_end = PhysAddr::new(align_down(dtb.as_usize(), PAGE_SIZE));
        }
    }

    if free_start > free_end {
        free_start = free_end;
    }

    MemoryMap {
        ram_start,
        ram_end,
        free_start,
        free_end,
        dtb_pa: PhysAddr::new(dtb_pa),
    }
}

fn sane_dram(start: PhysAddr, end: PhysAddr) -> bool {
    let s = start.as_usize();
    let e = end.as_usize();
    e > s
        && s >= DRAM_START
        && (e - s) >= 16 * 1024 * 1024
        && (e - s) <= 512 * 1024 * 1024
}
