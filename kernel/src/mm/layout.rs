//! Physical layout constants for QEMU virt + OpenSBI.
//!
//! Virt machine DRAM begins at 0x80000000. OpenSBI occupies the low part;
//! the kernel is linked at KERNEL_BASE (see linker.ld).

#![allow(dead_code)]

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SHIFT: usize = 12;

/* QEMU virt DRAM default when DTB walk yields nothing useful. */
pub const DRAM_START: usize = 0x8000_0000;
/* Match scripts/run-qemu.sh `-m 256M` fallback when DTB is missing. */
pub const DRAM_SIZE_DEFAULT: usize = 256 * 1024 * 1024;
pub const DRAM_END_DEFAULT: usize = DRAM_START + DRAM_SIZE_DEFAULT;

pub const KERNEL_BASE: usize = 0x8020_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(usize);

impl PhysAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }

    pub const fn align_down(self, align: usize) -> Self {
        Self(self.0 & !(align - 1))
    }

    pub const fn align_up(self, align: usize) -> Self {
        Self((self.0 + align - 1) & !(align - 1))
    }

    pub const fn page_index(self) -> usize {
        self.0 >> PAGE_SHIFT
    }
}

impl VirtAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn as_usize(self) -> usize {
        self.0
    }

    /*
     * Identity: under our early Sv39 map, VA == PA for DRAM.
     */
    pub const fn from_phys(pa: PhysAddr) -> Self {
        Self(pa.0)
    }
}

/*
 * Linker symbols — physical addresses while still identity-mapped / bare.
 */
pub fn skernel() -> PhysAddr {
    unsafe extern "C" {
        static __skernel: u8;
    }
    PhysAddr::new((&raw const __skernel) as usize)
}

pub fn ekernel() -> PhysAddr {
    unsafe extern "C" {
        static __ekernel: u8;
    }
    PhysAddr::new((&raw const __ekernel) as usize)
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

pub fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}
