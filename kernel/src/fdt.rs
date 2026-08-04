//! Flattened Device Tree walker (1.5) — shared platform discovery.
//!
//! Discovers memory, UART, virtio-mmio, and optional framebuffer nodes.
//! Does not drive devices; consumers (mm, block/virtio) read [`Platform`].

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::mm::layout::{align_up, PhysAddr};
use crate::println;

const FDT_MAGIC: u32 = 0xd00d_feed;
const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

const MAX_VIRTIO: usize = 8;
const MAX_NAME: usize = 64;

#[derive(Clone, Copy)]
pub struct Reg {
    pub base: usize,
    pub size: usize,
}

#[derive(Clone, Copy)]
pub struct VirtioMmio {
    pub reg: Reg,
    pub irq: u32,
}

#[derive(Clone, Copy)]
pub struct UartDev {
    pub reg: Reg,
    pub compatible: [u8; 32],
    pub compatible_len: usize,
}

#[derive(Clone, Copy)]
pub struct FramebufferHint {
    pub reg: Reg,
}

pub struct Platform {
    pub dtb_pa: usize,
    pub model: [u8; 64],
    pub model_len: usize,
    pub board_compat: [u8; 64],
    pub board_compat_len: usize,
    pub memory: Option<Reg>,
    pub uart: Option<UartDev>,
    pub virtio: [VirtioMmio; MAX_VIRTIO],
    pub virtio_count: usize,
    pub framebuffer: Option<FramebufferHint>,
}

static READY: AtomicBool = AtomicBool::new(false);
static DTB_PA: AtomicUsize = AtomicUsize::new(0);
static mut PLATFORM: Platform = Platform {
    dtb_pa: 0,
    model: [0; 64],
    model_len: 0,
    board_compat: [0; 64],
    board_compat_len: 0,
    memory: None,
    uart: None,
    virtio: [VirtioMmio {
        reg: Reg { base: 0, size: 0 },
        irq: 0,
    }; MAX_VIRTIO],
    virtio_count: 0,
    framebuffer: None,
};

/*
 * probe - walk DTB once and stash platform devices
 *
 * Safe to call after identity-map covers the DTB (it sits in DRAM).
 */
pub fn probe(dtb_pa: usize) {
    DTB_PA.store(dtb_pa, Ordering::Relaxed);
    let plat = unsafe { &mut *core::ptr::addr_of_mut!(PLATFORM) };
    *plat = Platform {
        dtb_pa,
        model: [0; 64],
        model_len: 0,
        board_compat: [0; 64],
        board_compat_len: 0,
        memory: None,
        uart: None,
        virtio: [VirtioMmio {
            reg: Reg { base: 0, size: 0 },
            irq: 0,
        }; MAX_VIRTIO],
        virtio_count: 0,
        framebuffer: None,
    };

    if dtb_pa == 0 {
        println!("fdt: no DTB (a1=0); using board fallbacks later");
        READY.store(true, Ordering::Relaxed);
        return;
    }

    if let Some(hdr) = unsafe { read_header(dtb_pa) } {
        if hdr.magic != FDT_MAGIC {
            println!("fdt: bad magic {:#x} at {:#x}", hdr.magic, dtb_pa);
            READY.store(true, Ordering::Relaxed);
            return;
        }
        println!(
            "fdt: blob pa={:#x} size={} version={}",
            dtb_pa, hdr.totalsize, hdr.version
        );
        walk(dtb_pa, &hdr, plat);
    } else {
        println!("fdt: cannot read header at {:#x}", dtb_pa);
    }

    READY.store(true, Ordering::Relaxed);
    log_summary(plat);
}

pub fn ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub fn get() -> Option<&'static Platform> {
    if !ready() {
        return None;
    }
    Some(unsafe { &*core::ptr::addr_of!(PLATFORM) })
}

pub fn dtb_pa() -> usize {
    DTB_PA.load(Ordering::Relaxed)
}

/*
 * memory_reg - first /memory reg, if discovered
 */
pub fn memory_reg() -> Option<(PhysAddr, PhysAddr)> {
    let p = get()?;
    let r = p.memory?;
    if r.size == 0 {
        return None;
    }
    Some((PhysAddr::new(r.base), PhysAddr::new(r.base + r.size)))
}

fn log_summary(p: &Platform) {
    if p.model_len > 0 {
        let m = core::str::from_utf8(&p.model[..p.model_len]).unwrap_or("?");
        println!("fdt: model \"{}\"", m);
    }
    if p.board_compat_len > 0 {
        /* First compatible string only (NUL-separated list). */
        let end = p.board_compat[..p.board_compat_len]
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(p.board_compat_len);
        let c = core::str::from_utf8(&p.board_compat[..end]).unwrap_or("?");
        println!("fdt: board {}", c);
    }
    if let Some(m) = p.memory {
        println!(
            "fdt: memory {:#x}..{:#x} ({} MiB)",
            m.base,
            m.base + m.size,
            m.size / (1024 * 1024)
        );
    } else {
        println!("fdt: memory node not found");
    }
    if let Some(u) = p.uart {
        let compat = core::str::from_utf8(&u.compatible[..u.compatible_len]).unwrap_or("?");
        println!(
            "fdt: uart {} @ {:#x} size={:#x}",
            compat, u.reg.base, u.reg.size
        );
    } else {
        println!("fdt: uart not found (console stays on SBI)");
    }
    println!("fdt: virtio-mmio count={}", p.virtio_count);
    for i in 0..p.virtio_count {
        let v = p.virtio[i];
        println!(
            "fdt:   virtio[{}] mmio={:#x} size={:#x} irq={}",
            i, v.reg.base, v.reg.size, v.irq
        );
    }
    if let Some(fb) = p.framebuffer {
        println!(
            "fdt: framebuffer hint @ {:#x} size={:#x}",
            fb.reg.base, fb.reg.size
        );
    }
}

struct Header {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    size_dt_struct: u32,
    version: u32,
}

unsafe fn read_header(pa: usize) -> Option<Header> {
    Some(Header {
        magic: read_u32_be(pa),
        totalsize: read_u32_be(pa + 4),
        off_dt_struct: read_u32_be(pa + 8),
        off_dt_strings: read_u32_be(pa + 12),
        size_dt_struct: read_u32_be(pa + 36),
        version: read_u32_be(pa + 20),
    })
}

fn walk(dtb_pa: usize, hdr: &Header, plat: &mut Platform) {
    let struct_off = dtb_pa + hdr.off_dt_struct as usize;
    let strings_off = dtb_pa + hdr.off_dt_strings as usize;
    let struct_end = struct_off + hdr.size_dt_struct as usize;

    let mut cells: [(u32, u32); 16] = [(2, 1); 16];
    let mut depth: usize = 0;
    let mut node_name = [0u8; MAX_NAME];
    let mut node_name_len = 0usize;
    let mut compat = [0u8; 64];
    let mut compat_len = 0usize;
    let mut reg: Option<Reg> = None;
    let mut irq: u32 = 0;
    let mut p = struct_off;

    while p + 4 <= struct_end {
        let token = unsafe { read_u32_be(p) };
        p += 4;
        match token {
            FDT_BEGIN_NODE => {
                let name = unsafe { cstr_at(p) };
                p = align_up(p + name.len() + 1, 4);
                if depth + 1 >= cells.len() {
                    break;
                }
                cells[depth + 1] = cells[depth];
                depth += 1;
                node_name_len = name.len().min(MAX_NAME - 1);
                node_name[..node_name_len].copy_from_slice(&name.as_bytes()[..node_name_len]);
                node_name[node_name_len] = 0;
                compat_len = 0;
                reg = None;
                irq = 0;
            }
            FDT_END_NODE => {
                if depth == 0 {
                    break;
                }
                finish_node(
                    plat,
                    &node_name[..node_name_len],
                    &compat[..compat_len],
                    reg,
                    irq,
                );
                depth -= 1;
                node_name_len = 0;
                compat_len = 0;
                reg = None;
                irq = 0;
            }
            FDT_PROP => {
                let len = unsafe { read_u32_be(p) } as usize;
                let nameoff = unsafe { read_u32_be(p + 4) } as usize;
                p += 8;
                let pname = unsafe { cstr_at(strings_off + nameoff) };
                let (ac, sc) = cells[depth];
                if pname == "#address-cells" && len == 4 {
                    cells[depth].0 = unsafe { read_u32_be(p) };
                } else if pname == "#size-cells" && len == 4 {
                    cells[depth].1 = unsafe { read_u32_be(p) };
                } else if pname == "reg" && reg.is_none() {
                    if let Some((base, size)) = unsafe { read_reg(p, ac, sc) } {
                        if size > 0 {
                            reg = Some(Reg { base, size });
                        }
                    }
                } else if pname == "model" && depth == 1 && len > 0 && plat.model_len == 0 {
                    let n = len.min(63);
                    unsafe {
                        core::ptr::copy_nonoverlapping(p as *const u8, plat.model.as_mut_ptr(), n);
                    }
                    let mut ml = n;
                    while ml > 0 && plat.model[ml - 1] == 0 {
                        ml -= 1;
                    }
                    plat.model_len = ml;
                } else if pname == "compatible" && len > 0 {
                    let n = len.min(63);
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            p as *const u8,
                            compat.as_mut_ptr(),
                            n,
                        );
                    }
                    /* NUL-separated list; keep whole blob for contains checks. */
                    compat_len = n;
                    while compat_len > 0 && compat[compat_len - 1] == 0 {
                        compat_len -= 1;
                    }
                    /* Root board compatible (empty node name, depth 1). */
                    if depth == 1 && node_name_len == 0 && plat.board_compat_len == 0 {
                        plat.board_compat[..n].copy_from_slice(&compat[..n]);
                        plat.board_compat_len = n;
                    }
                } else if pname == "interrupts" && len >= 4 {
                    irq = unsafe { read_u32_be(p) };
                }
                p = align_up(p + len, 4);
            }
            FDT_NOP => {}
            FDT_END => break,
            _ => break,
        }
    }
}

fn finish_node(
    plat: &mut Platform,
    name: &[u8],
    compat: &[u8],
    reg: Option<Reg>,
    irq: u32,
) {
    let name_str = core::str::from_utf8(name).unwrap_or("");
    let _ = core::str::from_utf8(compat);

    if name_str.starts_with("memory") {
        if plat.memory.is_none() {
            if let Some(r) = reg {
                plat.memory = Some(r);
            }
        }
        return;
    }

    if compat_contains(compat, b"virtio,mmio") {
        if let Some(r) = reg {
            if plat.virtio_count < MAX_VIRTIO {
                plat.virtio[plat.virtio_count] = VirtioMmio { reg: r, irq };
                plat.virtio_count += 1;
            }
        }
        return;
    }

    if plat.uart.is_none()
        && (compat_contains(compat, b"ns16550a")
            || compat_contains(compat, b"ns16550")
            || compat_contains(compat, b"8250"))
    {
        if let Some(r) = reg {
            let label: &[u8] = if compat_contains(compat, b"ns16550a") {
                b"ns16550a"
            } else if compat_contains(compat, b"ns16550") {
                b"ns16550"
            } else {
                b"8250"
            };
            let mut ucompat = [0u8; 32];
            ucompat[..label.len()].copy_from_slice(label);
            plat.uart = Some(UartDev {
                reg: r,
                compatible: ucompat,
                compatible_len: label.len(),
            });
        }
        return;
    }

    if plat.framebuffer.is_none()
        && (compat_contains(compat, b"simple-framebuffer")
            || compat_contains(compat, b"ramfb")
            || name_str.contains("framebuffer"))
    {
        if let Some(r) = reg {
            plat.framebuffer = Some(FramebufferHint { reg: r });
        }
    }
}

fn compat_contains(compat: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || compat.len() < needle.len() {
        return false;
    }
    compat.windows(needle.len()).any(|w| w == needle)
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
