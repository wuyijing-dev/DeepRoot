//! Kernel bump heap (0.2.2).
//!
//! Carve N frames at init and bump-allocate forever (no free). Enough for
//! early page tables and `alloc` until a real slab arrives.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::frame;
use super::layout::PAGE_SIZE;

struct BumpHeap {
    start: AtomicUsize,
    end: AtomicUsize,
    offset: AtomicUsize,
}

unsafe impl Sync for BumpHeap {}

static HEAP: BumpHeap = BumpHeap {
    start: AtomicUsize::new(0),
    end: AtomicUsize::new(0),
    offset: AtomicUsize::new(0),
};

/*
 * init - reserve `pages` contiguous frames for the bump heap
 * @pages: number of 4KiB frames to carve from the frame allocator
 */
pub fn init(pages: usize) {
    assert!(pages > 0);
    let first = frame::alloc().expect("heap: first frame");
    let mut last = first;
    for _ in 1..pages {
        let p = frame::alloc().expect("heap: frame");
        assert_eq!(
            p.as_usize(),
            last.as_usize() + PAGE_SIZE,
            "heap frames not contiguous — alloc from low addresses"
        );
        last = p;
    }
    let start = first.as_usize();
    let end = last.as_usize() + PAGE_SIZE;
    HEAP.start.store(start, Ordering::Relaxed);
    HEAP.end.store(end, Ordering::Relaxed);
    HEAP.offset.store(start, Ordering::Relaxed);
}

struct LockedBump;

/*
 * Safety: single-hart bump; GlobalAlloc contract upheld (aligned, within heap).
 */
unsafe impl GlobalAlloc for LockedBump {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mut cursor = HEAP.offset.load(Ordering::Relaxed);
        let end = HEAP.end.load(Ordering::Relaxed);
        cursor = (cursor + align - 1) & !(align - 1);
        let next = match cursor.checked_add(size) {
            Some(n) => n,
            None => return ptr::null_mut(),
        };
        if next > end {
            return ptr::null_mut();
        }
        HEAP.offset.store(next, Ordering::Relaxed);
        cursor as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        /* bump heap does not free */
    }
}

#[global_allocator]
static GLOBAL: LockedBump = LockedBump;

pub fn used() -> usize {
    HEAP.offset.load(Ordering::Relaxed) - HEAP.start.load(Ordering::Relaxed)
}

pub fn capacity() -> usize {
    HEAP.end.load(Ordering::Relaxed) - HEAP.start.load(Ordering::Relaxed)
}
