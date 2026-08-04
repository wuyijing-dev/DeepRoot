//! Per-task address space (1.1).

use super::layout::PhysAddr;
use super::sv39;

/// Userspace address space rooted at a Sv39 page-table.
pub struct AddrSpace {
    pub root_pa: PhysAddr,
}

impl AddrSpace {
    /*
     * create - clone kernel template root for a new task
     */
    pub fn create() -> Option<Self> {
        let root_pa = sv39::clone_user_root()?;
        Some(Self { root_pa })
    }

    pub fn root_ppn(&self) -> usize {
        self.root_pa.page_index()
    }

    pub fn activate(&self) {
        sv39::activate(self.root_pa);
    }

    pub fn map_user(&self, va: usize, pa: PhysAddr, exec: bool, write: bool) {
        sv39::map_user_in(self.root_pa, va, pa, exec, write);
    }
}

#[allow(unused_imports)]
pub use sv39::page_fault_hint;
