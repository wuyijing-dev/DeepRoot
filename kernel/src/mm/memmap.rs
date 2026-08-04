//! Physical memory map discovery (0.2.0).
//!
//! Walks a minimal subset of the Flattened Device Tree for a `memory@…`
//! `reg` property. Falls back to QEMU virt 128MiB defaults so worksheets
//! still run if the DTB walk fails a sanity check.

use super::layout::{
    align_down, align_up, ekernel, PhysAddr, DRAM_END_DEFAULT, DRAM_START, PAGE_SIZE,
};

#[allow(dead_code)]
pub struct MemoryMap {
    pub ram_start: PhysAddr,
    pub ram_end: PhysAddr,
    pub free_start: PhysAddr,
    pub free_end: PhysAddr,
    pub dtb_pa: PhysAddr,
}

/*
 * discover - build a MemoryMap from DTB or fallback constants
 * @dtb_pa: physical address of FDT blob (OpenSBI a1); 0 means unknown
 *
 * Free RAM is [ekernel, ram_end), shrunk if the DTB sits inside that range
 * so we never hand DTB pages to the frame allocator.
 */
pub fn discover(dtb_pa: usize) -> MemoryMap {
    let (ram_start, ram_end) = parse_memory_reg(dtb_pa)
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

/*
 * parse_memory_reg - tiny FDT walker for memory@* reg <addr,size>
 *
 * Tracks #address-cells / #size-cells on a depth stack so soc children do
 * not corrupt the root cell counts used by /memory.
 */
fn parse_memory_reg(dtb_pa: usize) -> Option<(PhysAddr, PhysAddr)> {
    if dtb_pa == 0 {
        return None;
    }
    let hdr = unsafe { read_fdt_header(dtb_pa)? };
    if hdr.magic != 0xd00d_feed {
        return None;
    }

    let struct_off = dtb_pa + hdr.off_dt_struct as usize;
    let strings_off = dtb_pa + hdr.off_dt_strings as usize;
    let struct_end = struct_off + hdr.size_dt_struct as usize;

    /* (addr_cells, size_cells) stack — index 0 is root defaults before props. */
    let mut cells: [(u32, u32); 16] = [(2, 1); 16];
    let mut depth: usize = 0;
    let mut in_memory = false;
    let mut p = struct_off;
    let mut found: Option<(PhysAddr, PhysAddr)> = None;

    while p + 4 <= struct_end {
        let token = unsafe { read_u32_be(p) };
        p += 4;
        match token {
            0x1 => {
                /* FDT_BEGIN_NODE */
                let name = unsafe { cstr_at(p) };
                p = align_up(p + name.len() + 1, 4);
                if depth + 1 >= cells.len() {
                    break;
                }
                /* inherit parent cells until this node overrides */
                cells[depth + 1] = cells[depth];
                depth += 1;
                in_memory = name.starts_with("memory");
            }
            0x2 => {
                /* FDT_END_NODE */
                if depth == 0 {
                    break;
                }
                depth -= 1;
                in_memory = false;
            }
            0x3 => {
                let len = unsafe { read_u32_be(p) } as usize;
                let nameoff = unsafe { read_u32_be(p + 4) } as usize;
                p += 8;
                let pname = unsafe { cstr_at(strings_off + nameoff) };
                let (ac, sc) = cells[depth];
                if pname == "#address-cells" && len == 4 {
                    cells[depth].0 = unsafe { read_u32_be(p) };
                } else if pname == "#size-cells" && len == 4 {
                    cells[depth].1 = unsafe { read_u32_be(p) };
                } else if in_memory && pname == "reg" && found.is_none() {
                    if let Some((base, size)) = unsafe { read_reg(p, ac, sc) } {
                        if size > 0 {
                            found = Some((PhysAddr::new(base), PhysAddr::new(base + size)));
                        }
                    }
                }
                let _ = ac;
                let _ = sc;
                p = align_up(p + len, 4);
            }
            0x4 => {}
            0x9 => break,
            _ => break,
        }
    }
    found
}

#[repr(C)]
struct FdtHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

unsafe fn read_fdt_header(pa: usize) -> Option<FdtHeader> {
    Some(FdtHeader {
        magic: read_u32_be(pa),
        totalsize: read_u32_be(pa + 4),
        off_dt_struct: read_u32_be(pa + 8),
        off_dt_strings: read_u32_be(pa + 12),
        off_mem_rsvmap: read_u32_be(pa + 16),
        version: read_u32_be(pa + 20),
        last_comp_version: read_u32_be(pa + 24),
        boot_cpuid_phys: read_u32_be(pa + 28),
        size_dt_strings: read_u32_be(pa + 32),
        size_dt_struct: read_u32_be(pa + 36),
    })
}

unsafe fn read_u32_be(pa: usize) -> u32 {
    let p = pa as *const u8;
    u32::from_be_bytes([*p, *p.add(1), *p.add(2), *p.add(3)])
}

unsafe fn read_u64_be(pa: usize) -> u64 {
    let hi = read_u32_be(pa) as u64;
    let lo = read_u32_be(pa + 4) as u64;
    (hi << 32) | lo
}

unsafe fn cstr_at(pa: usize) -> &'static str {
    let mut len = 0usize;
    let p = pa as *const u8;
    while *p.add(len) != 0 {
        len += 1;
        if len > 128 {
            break;
        }
    }
    core::str::from_utf8_unchecked(core::slice::from_raw_parts(p, len))
}

unsafe fn read_reg(pa: usize, addr_cells: u32, size_cells: u32) -> Option<(usize, usize)> {
    let mut off = 0usize;
    let base = match addr_cells {
        1 => {
            let v = read_u32_be(pa + off) as usize;
            off += 4;
            v
        }
        2 => {
            let v = read_u64_be(pa + off) as usize;
            off += 8;
            v
        }
        _ => return None,
    };
    let size = match size_cells {
        1 => read_u32_be(pa + off) as usize,
        2 => read_u64_be(pa + off) as usize,
        _ => return None,
    };
    Some((base, size))
}
