//! User address-space stubs (0.2.5).
//!
//! Full user page tables arrive with Server Grove / Schedule Canopy.
//! Here we only name the object learners will extend.

#![allow(dead_code)]

use super::layout::PhysAddr;
use super::sv39;

/// Placeholder for a userspace address space (root PPN + metadata).
pub struct AddrSpace {
    pub root_ppn: usize,
}

impl AddrSpace {
    /*
     * create - allocate an empty root page table for a future user task
     *
     * Returns None if the frame allocator is exhausted.
     */
    pub fn create() -> Option<Self> {
        let root = super::frame::alloc()?;
        Some(Self {
            root_ppn: root.page_index(),
        })
    }

    /*
     * destroy - free the root frame (does not walk children yet)
     *
     * Stub: leaking mid-level tables is intentional until 0.5.x.
     */
    pub fn destroy(self) {
        super::frame::free(PhysAddr::new(self.root_ppn << 12));
    }

    pub fn root_pa(&self) -> PhysAddr {
        PhysAddr::new(self.root_ppn << 12)
    }
}

/* Re-export fault hint so trap can stay thin. */
#[allow(unused_imports)]
pub use sv39::page_fault_hint;
