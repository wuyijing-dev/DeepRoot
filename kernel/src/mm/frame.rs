//! Bitmap frame allocator (0.2.1).
//!
//! Tracks free 4KiB frames in [free_start, free_end). Bitmap lives in BSS
//! sized for up to 256MiB DRAM so QEMU `-m` bumps stay simple.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::sync::SpinLock;

use super::layout::{PhysAddr, PAGE_SHIFT, PAGE_SIZE};

static FRAME_LOCK: SpinLock = SpinLock::new();

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
 * Safety: bitmap mutated only under FRAME_LOCK (1.7 SMP).
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
    alloc_contiguous(1)
}

/*
 * alloc_contiguous - take @count consecutive free frames (1..=512)
 */
pub fn alloc_contiguous(count: usize) -> Option<PhysAddr> {
    if count == 0 || count > 512 {
        return None;
    }
    let _g = FRAME_LOCK.lock();
    let n = ALLOC.nframes.load(Ordering::Relaxed);
    let words = unsafe { &mut *ALLOC.words.get() };
    let base_ppn = ALLOC.base_ppn.load(Ordering::Relaxed);
    'outer: for i in 0..=(n.saturating_sub(count)) {
        for j in 0..count {
            let idx = i + j;
            let w = idx / 64;
            let b = idx % 64;
            if words[w] & (1u64 << b) == 0 {
                continue 'outer;
            }
        }
        for j in 0..count {
            let idx = i + j;
            let w = idx / 64;
            let b = idx % 64;
            words[w] &= !(1u64 << b);
        }
        ALLOC.free.fetch_sub(count, Ordering::Relaxed);
        let pa = PhysAddr::new((base_ppn + i) << PAGE_SHIFT);
        for j in 0..count {
            zero_page(PhysAddr::new(pa.as_usize() + j * PAGE_SIZE));
        }
        return Some(pa);
    }
    None
}

/*
 * contains - true if @pa is a RAM page owned by this allocator
 *
 * Device MMIO Frame badges are CapType::Frame too but must never hit free().
 */
pub fn contains(pa: PhysAddr) -> bool {
    if pa.as_usize() % PAGE_SIZE != 0 {
        return false;
    }
    let ppn = pa.page_index();
    let base = ALLOC.base_ppn.load(Ordering::Relaxed);
    let n = ALLOC.nframes.load(Ordering::Relaxed);
    ppn >= base && ppn < base + n
}

/*
 * free - return a frame previously obtained from alloc()
 */
pub fn free(pa: PhysAddr) {
    free_contiguous(pa, 1);
}

/*
 * free_contiguous - return @count pages starting at @pa
 */
pub fn free_contiguous(pa: PhysAddr, count: usize) {
    if count == 0 {
        return;
    }
    let _g = FRAME_LOCK.lock();
    let base = ALLOC.base_ppn.load(Ordering::Relaxed);
    let n = ALLOC.nframes.load(Ordering::Relaxed);
    let words = unsafe { &mut *ALLOC.words.get() };
    for j in 0..count {
        let page = PhysAddr::new(pa.as_usize() + j * PAGE_SIZE);
        let ppn = page.page_index();
        assert!(ppn >= base && ppn < base + n);
        let i = ppn - base;
        let w = i / 64;
        let b = i % 64;
        let mask = 1u64 << b;
        assert!(words[w] & mask == 0, "double free frame");
        words[w] |= mask;
    }
    ALLOC.free.fetch_add(count, Ordering::Relaxed);
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
