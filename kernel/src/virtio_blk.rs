//! Virtio-mmio block driver (1.6) — QEMU virt **legacy** transport (version 1).
//!
//! Discovers a block device among FDT `virtio,mmio` nodes. Completions are
//! polled (no PLIC yet). Sector size is 512 bytes.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::fdt;
use crate::mm::layout::PAGE_SIZE;
use crate::mm::sv39;
use crate::println;

const MAGIC: u32 = 0x7472_6976;
const DEV_BLK: u32 = 2;

const MMIO_MAGIC: usize = 0x000;
const MMIO_VERSION: usize = 0x004;
const MMIO_DEVICE_ID: usize = 0x008;
const MMIO_HOST_FEATURES: usize = 0x010;
const MMIO_GUEST_FEATURES: usize = 0x020;
const MMIO_GUEST_PAGE_SIZE: usize = 0x028;
const MMIO_QUEUE_SEL: usize = 0x030;
const MMIO_QUEUE_NUM_MAX: usize = 0x034;
const MMIO_QUEUE_NUM: usize = 0x038;
const MMIO_QUEUE_ALIGN: usize = 0x03c;
const MMIO_QUEUE_PFN: usize = 0x040;
const MMIO_QUEUE_NOTIFY: usize = 0x050;
const MMIO_INTERRUPT_STATUS: usize = 0x060;
const MMIO_INTERRUPT_ACK: usize = 0x064;
const MMIO_STATUS: usize = 0x070;
const MMIO_CONFIG: usize = 0x100;

const S_ACKNOWLEDGE: u32 = 1;
const S_DRIVER: u32 = 2;
const S_DRIVER_OK: u32 = 4;

const VIRTIO_BLK_F_RO: u32 = 5;

const VRING_DESC_F_NEXT: u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;

pub const SECTOR_SIZE: usize = 512;
const QUEUE_NUM: usize = 8;

#[repr(C)]
#[derive(Clone, Copy)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
struct BlkReq {
    type_: u32,
    reserved: u32,
    sector: u64,
}

#[repr(C, align(4096))]
struct QueueMem {
    raw: [u8; PAGE_SIZE * 2],
}

struct Disk {
    mmio: usize,
    desc: *mut VirtqDesc,
    avail_idx: *mut u16,
    avail_ring: *mut u16,
    used_idx: *mut u16,
    free: [bool; QUEUE_NUM],
    last_used: u16,
    req: BlkReq,
    status: u8,
    sector_buf: [u8; SECTOR_SIZE],
}

static READY: AtomicBool = AtomicBool::new(false);
static CAP_SECTORS: AtomicUsize = AtomicUsize::new(0);
static mut QUEUE: QueueMem = QueueMem {
    raw: [0; PAGE_SIZE * 2],
};
static mut DISK: Disk = Disk {
    mmio: 0,
    desc: core::ptr::null_mut(),
    avail_idx: core::ptr::null_mut(),
    avail_ring: core::ptr::null_mut(),
    used_idx: core::ptr::null_mut(),
    free: [true; QUEUE_NUM],
    last_used: 0,
    req: BlkReq {
        type_: 0,
        reserved: 0,
        sector: 0,
    },
    status: 0xff,
    sector_buf: [0; SECTOR_SIZE],
};

fn r32(mmio: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((mmio + off) as *const u32) }
}

fn w32(mmio: usize, off: usize, v: u32) {
    unsafe { core::ptr::write_volatile((mmio + off) as *mut u32, v) }
}

pub fn init() -> bool {
    let Some(plat) = fdt::get() else {
        println!("virtio-blk: no platform (fdt not ready)");
        return false;
    };

    let mut found = None;
    for i in 0..plat.virtio_count {
        let v = plat.virtio[i];
        sv39::map_mmio_range(v.reg.base, v.reg.size.max(PAGE_SIZE));
        let mmio = v.reg.base;
        if r32(mmio, MMIO_MAGIC) != MAGIC {
            continue;
        }
        if r32(mmio, MMIO_DEVICE_ID) != DEV_BLK {
            continue;
        }
        let ver = r32(mmio, MMIO_VERSION);
        if ver != 1 {
            println!("virtio-blk: skip mmio={:#x} ver={} (need legacy v1)", mmio, ver);
            continue;
        }
        found = Some(mmio);
        break;
    }

    let Some(mmio) = found else {
        println!(
            "virtio-blk: no block device among {} mmio slots",
            plat.virtio_count
        );
        return false;
    };

    w32(mmio, MMIO_STATUS, 0);
    let mut status = S_ACKNOWLEDGE | S_DRIVER;
    w32(mmio, MMIO_STATUS, status);

    let host = r32(mmio, MMIO_HOST_FEATURES);
    w32(mmio, MMIO_GUEST_FEATURES, host & !(1 << VIRTIO_BLK_F_RO));
    w32(mmio, MMIO_GUEST_PAGE_SIZE, PAGE_SIZE as u32);

    w32(mmio, MMIO_QUEUE_SEL, 0);
    let max = r32(mmio, MMIO_QUEUE_NUM_MAX);
    if max < QUEUE_NUM as u32 {
        println!("virtio-blk: queue too small ({})", max);
        return false;
    }
    w32(mmio, MMIO_QUEUE_NUM, QUEUE_NUM as u32);
    w32(mmio, MMIO_QUEUE_ALIGN, PAGE_SIZE as u32);

    let qpa = unsafe { core::ptr::addr_of_mut!(QUEUE.raw) as usize };
    unsafe {
        core::ptr::write_bytes(qpa as *mut u8, 0, PAGE_SIZE * 2);
    }
    w32(mmio, MMIO_QUEUE_PFN, (qpa / PAGE_SIZE) as u32);

    let desc_sz = QUEUE_NUM * core::mem::size_of::<VirtqDesc>();
    let avail_off = desc_sz;
    let used_off = PAGE_SIZE; /* QueueAlign */

    let cap = unsafe { core::ptr::read_volatile((mmio + MMIO_CONFIG) as *const u64) };

    status |= S_DRIVER_OK;
    w32(mmio, MMIO_STATUS, status);

    let disk = unsafe { &mut *core::ptr::addr_of_mut!(DISK) };
    disk.mmio = mmio;
    disk.desc = qpa as *mut VirtqDesc;
    disk.avail_idx = (qpa + avail_off + 2) as *mut u16;
    disk.avail_ring = (qpa + avail_off + 4) as *mut u16;
    disk.used_idx = (qpa + used_off + 2) as *mut u16;
    disk.free = [true; QUEUE_NUM];
    disk.last_used = 0;

    CAP_SECTORS.store(cap as usize, Ordering::Relaxed);
    READY.store(true, Ordering::Relaxed);
    println!(
        "virtio-blk: ready mmio={:#x} capacity={} sectors ({} KiB) legacy",
        mmio,
        cap,
        (cap as usize) * SECTOR_SIZE / 1024
    );
    true
}

pub fn ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub fn capacity_bytes() -> usize {
    CAP_SECTORS.load(Ordering::Relaxed) * SECTOR_SIZE
}

fn alloc_desc(disk: &mut Disk) -> Option<usize> {
    for i in 0..QUEUE_NUM {
        if disk.free[i] {
            disk.free[i] = false;
            return Some(i);
        }
    }
    None
}

fn free_desc(disk: &mut Disk, i: usize) {
    unsafe {
        *disk.desc.add(i) = VirtqDesc {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        };
    }
    disk.free[i] = true;
}

fn free_chain(disk: &mut Disk, mut i: usize) {
    loop {
        let (flags, next) = unsafe {
            let d = &*disk.desc.add(i);
            (d.flags, d.next as usize)
        };
        free_desc(disk, i);
        if flags & VRING_DESC_F_NEXT == 0 {
            break;
        }
        i = next;
    }
}

pub fn rw_sector(sector: u64, buf: &mut [u8], write: bool) -> bool {
    if !ready() || buf.len() < SECTOR_SIZE {
        return false;
    }
    let cap = CAP_SECTORS.load(Ordering::Relaxed) as u64;
    if sector >= cap {
        return false;
    }

    let disk = unsafe { &mut *core::ptr::addr_of_mut!(DISK) };
    let mmio = disk.mmio;

    let i0 = match alloc_desc(disk) {
        Some(i) => i,
        None => return false,
    };
    let i1 = match alloc_desc(disk) {
        Some(i) => i,
        None => {
            free_desc(disk, i0);
            return false;
        }
    };
    let i2 = match alloc_desc(disk) {
        Some(i) => i,
        None => {
            free_desc(disk, i0);
            free_desc(disk, i1);
            return false;
        }
    };

    disk.req.type_ = if write {
        VIRTIO_BLK_T_OUT
    } else {
        VIRTIO_BLK_T_IN
    };
    disk.req.reserved = 0;
    disk.req.sector = sector;
    disk.status = 0xff;

    if write {
        disk.sector_buf.copy_from_slice(&buf[..SECTOR_SIZE]);
    }

    let req_pa = &disk.req as *const BlkReq as usize;
    let data_pa = disk.sector_buf.as_ptr() as usize;
    let st_pa = &disk.status as *const u8 as usize;

    unsafe {
        *disk.desc.add(i0) = VirtqDesc {
            addr: req_pa as u64,
            len: core::mem::size_of::<BlkReq>() as u32,
            flags: VRING_DESC_F_NEXT,
            next: i1 as u16,
        };
        *disk.desc.add(i1) = VirtqDesc {
            addr: data_pa as u64,
            len: SECTOR_SIZE as u32,
            flags: if write {
                VRING_DESC_F_NEXT
            } else {
                VRING_DESC_F_NEXT | VRING_DESC_F_WRITE
            },
            next: i2 as u16,
        };
        *disk.desc.add(i2) = VirtqDesc {
            addr: st_pa as u64,
            len: 1,
            flags: VRING_DESC_F_WRITE,
            next: 0,
        };

        let aidx = core::ptr::read_volatile(disk.avail_idx);
        core::ptr::write_volatile(
            disk.avail_ring.add((aidx as usize) % QUEUE_NUM),
            i0 as u16,
        );
        core::sync::atomic::fence(Ordering::SeqCst);
        core::ptr::write_volatile(disk.avail_idx, aidx.wrapping_add(1));
        core::sync::atomic::fence(Ordering::SeqCst);
    }
    w32(mmio, MMIO_QUEUE_NOTIFY, 0);

    let mut spins = 0u32;
    loop {
        core::sync::atomic::fence(Ordering::SeqCst);
        let uidx = unsafe { core::ptr::read_volatile(disk.used_idx) };
        if disk.last_used != uidx {
            break;
        }
        spins += 1;
        if spins > 50_000_000 {
            println!("virtio-blk: timeout sector={}", sector);
            free_chain(disk, i0);
            return false;
        }
        core::hint::spin_loop();
    }

    let isr = r32(mmio, MMIO_INTERRUPT_STATUS);
    w32(mmio, MMIO_INTERRUPT_ACK, isr & 3);

    while disk.last_used
        != unsafe { core::ptr::read_volatile(disk.used_idx) }
    {
        core::sync::atomic::fence(Ordering::SeqCst);
        disk.last_used = disk.last_used.wrapping_add(1);
    }

    let ok = disk.status == 0;
    if ok && !write {
        buf[..SECTOR_SIZE].copy_from_slice(&disk.sector_buf);
    }
    free_chain(disk, i0);
    ok
}

pub fn read_bytes(off: usize, out: &mut [u8]) -> usize {
    if !ready() || out.is_empty() {
        return 0;
    }
    let cap = capacity_bytes();
    if off >= cap {
        return 0;
    }
    let want = out.len().min(cap - off);
    let mut done = 0usize;
    while done < want {
        let abs = off + done;
        let sector = (abs / SECTOR_SIZE) as u64;
        let so = abs % SECTOR_SIZE;
        let mut tmp = [0u8; SECTOR_SIZE];
        if !rw_sector(sector, &mut tmp, false) {
            break;
        }
        let n = (SECTOR_SIZE - so).min(want - done);
        out[done..done + n].copy_from_slice(&tmp[so..so + n]);
        done += n;
    }
    done
}

pub fn write_bytes(off: usize, data: &[u8]) -> usize {
    if !ready() || data.is_empty() {
        return 0;
    }
    let cap = capacity_bytes();
    if off >= cap {
        return 0;
    }
    let want = data.len().min(cap - off);
    let mut done = 0usize;
    while done < want {
        let abs = off + done;
        let sector = (abs / SECTOR_SIZE) as u64;
        let so = abs % SECTOR_SIZE;
        let mut tmp = [0u8; SECTOR_SIZE];
        if so != 0 || (want - done) < SECTOR_SIZE {
            if !rw_sector(sector, &mut tmp, false) {
                break;
            }
        }
        let n = (SECTOR_SIZE - so).min(want - done);
        tmp[so..so + n].copy_from_slice(&data[done..done + n]);
        if !rw_sector(sector, &mut tmp, true) {
            break;
        }
        done += n;
    }
    done
}
