//! Bitmap frame allocator (0.2.1).
//!
//! Tracks free 4KiB frames in [free_start, free_end). Bitmap lives in BSS
//! sized for up to 256MiB DRAM so QEMU `-m` bumps stay simple.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::layout::{PhysAddr, PAGE_SHIFT, PAGE_SIZE};

/* 256 MiB / 4 KiB = 65536 frames; bitmap = 65536 bits = 8192 bytes. */
const MAX_FRAMES: usize = 65536;
const BITMAP_WORDS: usize = MAX_FRAMES / 64;

struct Allocator {
    words: UnsafeCell<[u64; BITMAP_WORDS]>,
    base_ppn: AtomicUsize,
    nframes: AtomicUsize,
    free: AtomicUsize,
}

/*
 * Safety: Address Sapling is single-hart; no IRQ nesting around alloc yet.
 */
unsafe impl Sync for Allocator {}

static ALLOC: Allocator = Allocator {
    words: UnsafeCell::new([0; BITMAP_WORDS]),
    base_ppn: AtomicUsize::new(0),
    nframes: AtomicUsize::new(0),
    free: AtomicUsize::new(0),
};

/*
 * init - mark all frames in [start, end) free
 * @start/@end: page-aligned physical range from MemoryMap
 */
pub fn init(start: PhysAddr, end: PhysAddr) {
    let start_ppn = start.page_index();
    let end_ppn = end.page_index();
    assert!(end_ppn >= start_ppn);
    let n = end_ppn - start_ppn;
    assert!(n <= MAX_FRAMES, "frame bitmap too small for RAM");

    ALLOC.base_ppn.store(start_ppn, Ordering::Relaxed);
    ALLOC.nframes.store(n, Ordering::Relaxed);
    ALLOC.free.store(n, Ordering::Relaxed);

    let words = unsafe { &mut *ALLOC.words.get() };
    words.fill(0);
    /* 1-bit = free. Set all n bits. */
    for i in 0..n {
        let w = i / 64;
        let b = i % 64;
        words[w] |= 1u64 << b;
    }
}

/*
 * alloc - take one free frame; returns physical address of page or None
 */
pub fn alloc() -> Option<PhysAddr> {
    let n = ALLOC.nframes.load(Ordering::Relaxed);
    let words = unsafe { &mut *ALLOC.words.get() };
    for i in 0..n {
        let w = i / 64;
        let b = i % 64;
        let mask = 1u64 << b;
        if words[w] & mask != 0 {
            words[w] &= !mask;
            ALLOC.free.fetch_sub(1, Ordering::Relaxed);
            let ppn = ALLOC.base_ppn.load(Ordering::Relaxed) + i;
            let pa = PhysAddr::new(ppn << PAGE_SHIFT);
            zero_page(pa);
            return Some(pa);
        }
    }
    None
}

/*
 * free - return a frame previously obtained from alloc()
 */
pub fn free(pa: PhysAddr) {
    let ppn = pa.page_index();
    let base = ALLOC.base_ppn.load(Ordering::Relaxed);
    let n = ALLOC.nframes.load(Ordering::Relaxed);
    assert!(ppn >= base && ppn < base + n);
    let i = ppn - base;
    let words = unsafe { &mut *ALLOC.words.get() };
    let w = i / 64;
    let b = i % 64;
    let mask = 1u64 << b;
    assert!(words[w] & mask == 0, "double free frame");
    words[w] |= mask;
    ALLOC.free.fetch_add(1, Ordering::Relaxed);
}

pub fn stats() -> (usize, usize) {
    (
        ALLOC.nframes.load(Ordering::Relaxed),
        ALLOC.free.load(Ordering::Relaxed),
    )
}

fn zero_page(pa: PhysAddr) {
    unsafe {
        let p = pa.as_usize() as *mut u8;
        core::ptr::write_bytes(p, 0, PAGE_SIZE);
    }
}
