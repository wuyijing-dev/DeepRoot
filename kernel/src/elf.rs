//! Minimal ELF64 loader for ET_EXEC RISC-V userspace servers (0.5.0).

use crate::mm::frame;
use crate::mm::layout::{PhysAddr, PAGE_SIZE};
use crate::mm::sv39;
use crate::println;

const PT_LOAD: u32 = 1;
const EM_RISCV: u16 = 243;
const ET_EXEC: u16 = 2;
const PF_X: u32 = 1;
const PF_W: u32 = 2;

#[repr(C)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct LoadedElf {
    pub entry: usize,
    #[allow(dead_code)]
    pub load_base: usize,
    #[allow(dead_code)]
    pub load_end: usize,
}

#[derive(Clone, Copy)]
struct PageSlot {
    va: usize,
    pa: PhysAddr,
    exec: bool,
    write: bool,
}

const MAX_PAGES: usize = 512;

/*
 * load - parse ELF, coalesce PT_LOAD onto shared pages, map U|perms
 *
 * Multiple LOAD headers often share one 4KiB page (.text + .rodata). We must
 * OR permission bits and copy into a single frame — remapping R-only over
 * R-X was causing instruction page faults.
 */
pub fn load(name: &str, bytes: &[u8]) -> Option<LoadedElf> {
    if bytes.len() < core::mem::size_of::<Elf64Ehdr>() || &bytes[0..4] != b"\x7fELF" {
        println!("elf: {} bad header", name);
        return None;
    }
    let ehdr = unsafe { &*(bytes.as_ptr() as *const Elf64Ehdr) };
    if ehdr.e_type != ET_EXEC || ehdr.e_machine != EM_RISCV {
        println!("elf: {} not RISC-V ET_EXEC", name);
        return None;
    }

    let mut pages: [Option<PageSlot>; MAX_PAGES] = [None; MAX_PAGES];
    let mut load_base = usize::MAX;
    let mut load_end = 0usize;

    for i in 0..ehdr.e_phnum as usize {
        let off = ehdr.e_phoff as usize + i * ehdr.e_phentsize as usize;
        if off + core::mem::size_of::<Elf64Phdr>() > bytes.len() {
            return None;
        }
        let ph = unsafe { &*(bytes.as_ptr().add(off) as *const Elf64Phdr) };
        if ph.p_type != PT_LOAD || ph.p_memsz == 0 {
            continue;
        }

        let va = ph.p_vaddr as usize;
        let memsz = ph.p_memsz as usize;
        let filesz = ph.p_filesz as usize;
        let file_off = ph.p_offset as usize;
        let exec = ph.p_flags & PF_X != 0;
        let write = ph.p_flags & PF_W != 0;

        let start = va & !(PAGE_SIZE - 1);
        let end = (va + memsz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        load_base = load_base.min(start);
        load_end = load_end.max(end);

        let mut page_va = start;
        while page_va < end {
            let slot_idx = match pages.iter().position(|s| s.as_ref().map(|p| p.va) == Some(page_va))
            {
                Some(i) => i,
                None => {
                    let free = pages.iter().position(|s| s.is_none())?;
                    let pa = frame::alloc()?;
                    pages[free] = Some(PageSlot {
                        va: page_va,
                        pa,
                        exec: false,
                        write: false,
                    });
                    free
                }
            };
            let slot = pages[slot_idx].as_mut().unwrap();
            slot.exec |= exec;
            slot.write |= write;
            let pa = slot.pa;

            for j in 0..PAGE_SIZE {
                let vaddr = page_va + j;
                if vaddr >= va && vaddr < va + filesz {
                    let src = file_off + (vaddr - va);
                    if src < bytes.len() {
                        unsafe {
                            *((pa.as_usize() + j) as *mut u8) = bytes[src];
                        }
                    }
                }
            }
            page_va += PAGE_SIZE;
        }
    }

    if load_base == usize::MAX {
        println!("elf: {} has no PT_LOAD", name);
        return None;
    }

    for slot in pages.iter().flatten() {
        sv39::map_user(slot.va, slot.pa, slot.exec, slot.write);
    }

    Some(LoadedElf {
        entry: ehdr.e_entry as usize,
        load_base,
        load_end,
    })
}

/*
 * load_into - like load, but map pages into @aspace
 */
pub fn load_into(
    aspace: &crate::mm::aspace::AddrSpace,
    name: &str,
    bytes: &[u8],
) -> Option<LoadedElf> {
    if bytes.len() < core::mem::size_of::<Elf64Ehdr>() || &bytes[0..4] != b"\x7fELF" {
        println!("elf: {} bad header", name);
        return None;
    }
    let ehdr = unsafe { &*(bytes.as_ptr() as *const Elf64Ehdr) };
    if ehdr.e_type != ET_EXEC || ehdr.e_machine != EM_RISCV {
        println!("elf: {} not RISC-V ET_EXEC", name);
        return None;
    }

    let mut pages: [Option<PageSlot>; MAX_PAGES] = [None; MAX_PAGES];
    let mut load_base = usize::MAX;
    let mut load_end = 0usize;

    for i in 0..ehdr.e_phnum as usize {
        let off = ehdr.e_phoff as usize + i * ehdr.e_phentsize as usize;
        if off + core::mem::size_of::<Elf64Phdr>() > bytes.len() {
            return None;
        }
        let ph = unsafe { &*(bytes.as_ptr().add(off) as *const Elf64Phdr) };
        if ph.p_type != PT_LOAD || ph.p_memsz == 0 {
            continue;
        }

        let va = ph.p_vaddr as usize;
        let memsz = ph.p_memsz as usize;
        let filesz = ph.p_filesz as usize;
        let file_off = ph.p_offset as usize;
        let exec = ph.p_flags & PF_X != 0;
        let write = ph.p_flags & PF_W != 0;

        let start = va & !(PAGE_SIZE - 1);
        let end = (va + memsz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        load_base = load_base.min(start);
        load_end = load_end.max(end);

        let mut page_va = start;
        while page_va < end {
            let slot_idx = match pages.iter().position(|s| s.as_ref().map(|p| p.va) == Some(page_va))
            {
                Some(i) => i,
                None => {
                    let free = pages.iter().position(|s| s.is_none())?;
                    let pa = frame::alloc()?;
                    pages[free] = Some(PageSlot {
                        va: page_va,
                        pa,
                        exec: false,
                        write: false,
                    });
                    free
                }
            };
            let slot = pages[slot_idx].as_mut().unwrap();
            slot.exec |= exec;
            slot.write |= write;
            let pa = slot.pa;

            for j in 0..PAGE_SIZE {
                let vaddr = page_va + j;
                if vaddr >= va && vaddr < va + filesz {
                    let src = file_off + (vaddr - va);
                    if src < bytes.len() {
                        unsafe {
                            *((pa.as_usize() + j) as *mut u8) = bytes[src];
                        }
                    }
                }
            }
            page_va += PAGE_SIZE;
        }
    }

    if load_base == usize::MAX {
        println!("elf: {} has no PT_LOAD", name);
        return None;
    }

    for slot in pages.iter().flatten() {
        aspace.map_user(slot.va, slot.pa, slot.exec, slot.write);
    }

    Some(LoadedElf {
        entry: ehdr.e_entry as usize,
        load_base,
        load_end,
    })
}
