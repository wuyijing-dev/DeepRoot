//! virtioblk — userspace virtio-mmio block driver on peel disk (1.16).
//!
//! Claims the **second** FDT virtio-blk (QEMU `virtio-mmio-bus.1` / hd1).
//! Kernel DRFS keeps the first. Completions use SYS_IRQ_WAIT (PLIC) with a
//! short poll fallback so smoke still passes if an IRQ is coalesced.

#![no_std]
#![no_main]

use deeproot_user::sys;

core::arch::global_asm!(
    r#"
    .section .text.entry, "ax"
    .globl _start
_start:
    la t0, __bss_start
    la t1, __bss_end
1:
    bgeu t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:
    call main
    li a0, 0
    li a7, 9
    ecall
3:
    wfi
    j 3b
"#
);

const MAGIC: u32 = 0x7472_6976;
const DEV_BLK: u32 = 2;
const SECTOR_SIZE: usize = 512;
const QUEUE_NUM: usize = 8;

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

const PAGE: usize = sys::PAGE_SIZE;
const QUEUE_VA: usize = sys::DMA_VA;
const DATA_VA: usize = sys::DMA_VA + 2 * PAGE;

#[repr(C)]
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

fn r32(mmio: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((mmio + off) as *const u32) }
}

fn w32(mmio: usize, off: usize, v: u32) {
    unsafe { core::ptr::write_volatile((mmio + off) as *mut u32, v) }
}

fn write_dec(n: u32) {
    let mut buf = [0u8; 12];
    let mut x = n;
    let mut i = 11usize;
    if x == 0 {
        buf[i] = b'0';
        i -= 1;
    } else {
        while x > 0 && i > 0 {
            buf[i] = b'0' + (x % 10) as u8;
            x /= 10;
            i -= 1;
        }
    }
    let _ = sys::debug_write_bytes(&buf[i + 1..]);
}

fn find_second_blk() -> Option<(usize, usize)> {
    let mut seen = 0u32;
    let mut i = 0usize;
    while i < 8 {
        let slot = sys::mmio_virtio(i);
        if slot < 0 {
            i += 1;
            continue;
        }
        if sys::frame_map(slot as usize, sys::MMIO_VA, true) < 0 {
            i += 1;
            continue;
        }
        let magic = r32(sys::MMIO_VA, MMIO_MAGIC);
        let did = r32(sys::MMIO_VA, MMIO_DEVICE_ID);
        let ver = r32(sys::MMIO_VA, MMIO_VERSION);
        if magic == MAGIC && did == DEV_BLK && ver == 1 {
            seen += 1;
            if seen == 2 {
                return Some((i, slot as usize));
            }
        }
        let _ = sys::frame_unmap(sys::MMIO_VA);
        i += 1;
    }
    None
}

struct Disk {
    mmio: usize,
    queue_pa: usize,
    data_pa: usize,
    desc: *mut VirtqDesc,
    avail_idx: *mut u16,
    avail_ring: *mut u16,
    used_idx: *mut u16,
    free: [bool; QUEUE_NUM],
    last_used: u16,
    capacity: u64,
    irq_slot: isize,
}

fn setup_queue(mmio: usize, queue_pa: usize, data_pa: usize, irq_slot: isize) -> Option<Disk> {
    w32(mmio, MMIO_STATUS, 0);
    let mut status = S_ACKNOWLEDGE | S_DRIVER;
    w32(mmio, MMIO_STATUS, status);

    let host = r32(mmio, MMIO_HOST_FEATURES);
    w32(mmio, MMIO_GUEST_FEATURES, host & !(1 << VIRTIO_BLK_F_RO));
    w32(mmio, MMIO_GUEST_PAGE_SIZE, PAGE as u32);

    w32(mmio, MMIO_QUEUE_SEL, 0);
    let max = r32(mmio, MMIO_QUEUE_NUM_MAX);
    if max < QUEUE_NUM as u32 {
        let _ = sys::debug_write("virtioblk: queue too small\n");
        return None;
    }
    w32(mmio, MMIO_QUEUE_NUM, QUEUE_NUM as u32);
    w32(mmio, MMIO_QUEUE_ALIGN, PAGE as u32);

    unsafe {
        core::ptr::write_bytes(QUEUE_VA as *mut u8, 0, PAGE * 2);
        core::ptr::write_bytes(DATA_VA as *mut u8, 0, PAGE);
    }
    w32(mmio, MMIO_QUEUE_PFN, (queue_pa / PAGE) as u32);

    let desc_sz = QUEUE_NUM * core::mem::size_of::<VirtqDesc>();
    let avail_off = desc_sz;
    let used_off = PAGE;
    let cap = unsafe { core::ptr::read_volatile((mmio + MMIO_CONFIG) as *const u64) };

    status |= S_DRIVER_OK;
    w32(mmio, MMIO_STATUS, status);

    Some(Disk {
        mmio,
        queue_pa,
        data_pa,
        desc: QUEUE_VA as *mut VirtqDesc,
        avail_idx: (QUEUE_VA + avail_off + 2) as *mut u16,
        avail_ring: (QUEUE_VA + avail_off + 4) as *mut u16,
        used_idx: (QUEUE_VA + used_off + 2) as *mut u16,
        free: [true; QUEUE_NUM],
        last_used: 0,
        capacity: cap,
        irq_slot,
    })
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

fn rw_sector(disk: &mut Disk, sector: u64, buf: &mut [u8], write: bool) -> bool {
    if buf.len() < SECTOR_SIZE || sector >= disk.capacity {
        return false;
    }
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

    /* Layout in DATA page: BlkReq | status | sector[512] */
    let req_va = DATA_VA;
    let st_va = DATA_VA + 16;
    let data_va = DATA_VA + 32;
    let req_pa = disk.data_pa;
    let st_pa = disk.data_pa + 16;
    let data_pa = disk.data_pa + 32;

    unsafe {
        let req = req_va as *mut BlkReq;
        (*req).type_ = if write {
            VIRTIO_BLK_T_OUT
        } else {
            VIRTIO_BLK_T_IN
        };
        (*req).reserved = 0;
        (*req).sector = sector;
        core::ptr::write_volatile(st_va as *mut u8, 0xff);
        if write {
            core::ptr::copy_nonoverlapping(buf.as_ptr(), data_va as *mut u8, SECTOR_SIZE);
        }

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
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        core::ptr::write_volatile(disk.avail_idx, aidx.wrapping_add(1));
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }
    w32(disk.mmio, MMIO_QUEUE_NOTIFY, 0);

    /* Brief poll, then SYS_IRQ_WAIT, then poll again (1.16). */
    let mut spins = 0u32;
    let mut waited = false;
    loop {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        let uidx = unsafe { core::ptr::read_volatile(disk.used_idx) };
        if disk.last_used != uidx {
            break;
        }
        if !waited && disk.irq_slot >= 0 && spins > 64 {
            let _ = sys::irq_wait(disk.irq_slot as usize);
            waited = true;
            spins = 0;
            continue;
        }
        spins += 1;
        if spins > 50_000_000 {
            free_chain(disk, i0);
            return false;
        }
        if spins & 0xff == 0 {
            let _ = sys::yield_now();
        } else {
            core::hint::spin_loop();
        }
    }
    if waited {
        let _ = sys::debug_write("virtioblk: irq wait ok\n");
    }

    let isr = r32(disk.mmio, MMIO_INTERRUPT_STATUS);
    w32(disk.mmio, MMIO_INTERRUPT_ACK, isr & 3);
    while disk.last_used != unsafe { core::ptr::read_volatile(disk.used_idx) } {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        disk.last_used = disk.last_used.wrapping_add(1);
    }

    let status = unsafe { core::ptr::read_volatile(st_va as *const u8) };
    let ok = status == 0;
    if ok && !write {
        unsafe {
            core::ptr::copy_nonoverlapping(data_va as *const u8, buf.as_mut_ptr(), SECTOR_SIZE);
        }
    }
    free_chain(disk, i0);
    ok
}

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("virtioblk: probe start\n");

    let Some((idx, mmio_slot)) = find_second_blk() else {
        let _ = sys::debug_write("virtioblk: no second block device\n");
        loop {
            let _ = sys::yield_now();
        }
    };

    let _ = sys::debug_write("virtioblk: claim idx=");
    write_dec(idx as u32);
    let _ = sys::debug_write("\n");

    let irq_slot = sys::irq_virtio(idx);
    if irq_slot >= 0 {
        let _ = sys::debug_write("virtioblk: irq cap\n");
    } else {
        let _ = sys::debug_write("virtioblk: irq mint failed\n");
    }

    let qslot = sys::frame_alloc_n(2);
    let dslot = sys::frame_alloc();
    if qslot < 0 || dslot < 0 {
        let _ = sys::debug_write("virtioblk: dma alloc failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    let queue_pa = sys::frame_phys(qslot as usize);
    let data_pa = sys::frame_phys(dslot as usize);
    if queue_pa < 0 || data_pa < 0 {
        let _ = sys::debug_write("virtioblk: frame_phys failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    if sys::frame_map(qslot as usize, QUEUE_VA, true) < 0
        || sys::frame_map(dslot as usize, DATA_VA, true) < 0
    {
        let _ = sys::debug_write("virtioblk: dma map failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }

    /* MMIO still mapped at MMIO_VA from find_second_blk. */
    let _ = mmio_slot;
    let Some(mut disk) = setup_queue(sys::MMIO_VA, queue_pa as usize, data_pa as usize, irq_slot)
    else {
        loop {
            let _ = sys::yield_now();
        }
    };
    let _ = sys::debug_write("virtioblk: ready sectors=");
    write_dec(disk.capacity as u32);
    let _ = sys::debug_write("\n");

    let mut buf = [0u8; SECTOR_SIZE];
    let magic = b"DeepRoot peel 1.14.3\n";
    for (i, b) in magic.iter().enumerate() {
        buf[i] = *b;
    }
    /* Use sector 1 — leave 0 free for future FS images. */
    if !rw_sector(&mut disk, 1, &mut buf, true) {
        let _ = sys::debug_write("virtioblk: write failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    buf.fill(0);
    if !rw_sector(&mut disk, 1, &mut buf, false) {
        let _ = sys::debug_write("virtioblk: read failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    let mut ok = true;
    for (i, b) in magic.iter().enumerate() {
        if buf[i] != *b {
            ok = false;
            break;
        }
    }
    if ok {
        let _ = sys::debug_write("virtioblk: rw ok\n");
        let _ = sys::debug_write("virtioblk: probe ok\n");
    } else {
        let _ = sys::debug_write("virtioblk: rw mismatch\n");
    }
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("virtioblk: PANIC\n");
    sys::exit(1);
}
