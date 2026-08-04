//! Memory management — Address Sapling (0.2.x).
//!
//! Layout discovery → frame allocator → bump heap → Sv39 identity map.

pub mod aspace;
pub mod frame;
pub mod heap;
pub mod layout;
pub mod memmap;
pub mod page;
pub mod sv39;

use crate::println;
use deeproot_abi::LedgerKind;
use layout::PAGE_SIZE;

/*
 * init - bring up physical memory, heap, and Sv39 identity mapping
 * @hartid: boot hart id from OpenSBI (a0)
 * @dtb_pa: physical address of DTB from OpenSBI (a1), may be 0
 *
 * Order matters: discover free RAM, init frames, carve heap pages, build
 * page tables with the frame allocator, then write satp.
 */
pub fn init(hartid: usize, dtb_pa: usize) {
    let map = memmap::discover(dtb_pa);
    println!(
        "mm: hart={} dtb={:#x} ram={:#x}..{:#x} free={:#x}..{:#x}",
        hartid,
        dtb_pa,
        map.ram_start.as_usize(),
        map.ram_end.as_usize(),
        map.free_start.as_usize(),
        map.free_end.as_usize()
    );

    frame::init(map.free_start, map.free_end);
    let (total, free) = frame::stats();
    println!(
        "mm: frames total={} free={} ({} KiB free)",
        total,
        free,
        free * PAGE_SIZE / 1024
    );

    heap::init(64); /* 64 pages = 256 KiB bump heap */
    println!(
        "mm: bump heap {} / {} bytes",
        heap::used(),
        heap::capacity()
    );

    crate::ledger::LEDGER.record(
        LedgerKind::Boot,
        map.free_start.as_usize() as u32,
        map.free_end.as_usize() as u32,
        free as u32,
    );

    sv39::init_identity(&map);
    println!("mm: Sv39 identity map active");
}
