//! Sv39 page tables + identity map (0.2.3 / 0.2.4) + user maps (0.5.x).

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use super::frame;
use super::layout::{PhysAddr, VirtAddr, PAGE_SIZE};
use super::memmap::MemoryMap;

const SATP_MODE_SV39: usize = 8;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct Pte(u64);

impl Pte {
    const V: u64 = 1 << 0;
    const R: u64 = 1 << 1;
    const W: u64 = 1 << 2;
    const X: u64 = 1 << 3;
    const U: u64 = 1 << 4;
    const A: u64 = 1 << 6;
    const D: u64 = 1 << 7;

    const fn is_valid(self) -> bool {
        self.0 & Self::V != 0
    }

    const fn is_leaf(self) -> bool {
        self.0 & (Self::R | Self::W | Self::X) != 0
    }

    const fn ppn(self) -> usize {
        ((self.0 >> 10) & ((1 << 44) - 1)) as usize
    }

    const fn from_ppn_flags(ppn: usize, flags: u64) -> Self {
        Self(((ppn as u64) << 10) | flags)
    }
}

#[repr(C, align(4096))]
struct PageTable {
    entries: [Pte; 512],
}

impl PageTable {
    fn from_pa(pa: PhysAddr) -> &'static mut Self {
        unsafe { &mut *(pa.as_usize() as *mut Self) }
    }
}

fn vpn(va: usize, level: usize) -> usize {
    (va >> (12 + 9 * level)) & 0x1ff
}

struct RootHolder {
    pa: AtomicUsize,
    table: UnsafeCell<usize>,
}

unsafe impl Sync for RootHolder {}

static ROOT: RootHolder = RootHolder {
    pa: AtomicUsize::new(0),
    table: UnsafeCell::new(0),
};

fn map_page(root: &mut PageTable, va: VirtAddr, pa: PhysAddr, flags: u64) {
    assert!(va.as_usize() % PAGE_SIZE == 0);
    assert!(pa.as_usize() % PAGE_SIZE == 0);

    let mut table = root as *mut PageTable;
    for level in (1..=2).rev() {
        let idx = vpn(va.as_usize(), level);
        let pte = unsafe { &mut (*table).entries[idx] };
        if !pte.is_valid() {
            let frame = frame::alloc().expect("sv39: mid-level table");
            *pte = Pte::from_ppn_flags(frame.page_index(), Pte::V);
        }
        assert!(!pte.is_leaf(), "sv39: unexpected leaf at mid level");
        table = PageTable::from_pa(PhysAddr::new(pte.ppn() << 12)) as *mut PageTable;
    }

    let idx = vpn(va.as_usize(), 0);
    let pte = unsafe { &mut (*table).entries[idx] };
    *pte = Pte::from_ppn_flags(pa.page_index(), flags | Pte::V | Pte::A | Pte::D);
}

pub fn init_identity(map: &MemoryMap) {
    let root_pa = frame::alloc().expect("sv39: root table");
    let root = PageTable::from_pa(root_pa);

    let mut pa = map.ram_start.as_usize();
    let end = map.ram_end.as_usize();
    let flags = Pte::R | Pte::W | Pte::X;
    while pa < end {
        map_page(root, VirtAddr::new(pa), PhysAddr::new(pa), flags);
        pa += PAGE_SIZE;
    }

    ROOT.pa.store(root_pa.as_usize(), Ordering::Relaxed);
    unsafe {
        *ROOT.table.get() = root_pa.as_usize();
    }
    activate(root_pa);
}

fn root_mut() -> &'static mut PageTable {
    let pa = ROOT.pa.load(Ordering::Relaxed);
    assert!(pa != 0);
    PageTable::from_pa(PhysAddr::new(pa))
}

/*
 * map_user - map one user page VA→PA with U|perms
 */
pub fn map_user(va: usize, pa: PhysAddr, exec: bool, write: bool) {
    let mut flags = Pte::R | Pte::U;
    if write {
        flags |= Pte::W;
    }
    if exec {
        flags |= Pte::X;
    }
    map_page(root_mut(), VirtAddr::new(va), pa, flags);
    unsafe {
        core::arch::asm!("sfence.vma", options(nostack));
    }
}

fn activate(root_pa: PhysAddr) {
    let satp = (SATP_MODE_SV39 << 60) | root_pa.page_index();
    unsafe {
        core::arch::asm!(
            "csrw satp, {satp}",
            "sfence.vma",
            satp = in(reg) satp,
            options(nostack),
        );
    }
}

pub fn page_fault_hint(stval: usize) -> &'static str {
    if stval == 0 {
        "null pointer?"
    } else {
        "unmapped VA — check identity map / user mappings"
    }
}
