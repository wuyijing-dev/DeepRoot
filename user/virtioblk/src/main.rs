//! virtioblk — userspace virtio-mmio probe (1.14.2 peel).
//!
//! Maps FDT virtio-mmio pages via `SYS_MMIO_VIRTIO` + `FRAME_MAP`, reads
//! magic / device_id. Does **not** claim the queue — kernel virtio-blk
//! still owns DRFS I/O until a later peel lands the full driver.

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

const MAGIC: u32 = 0x7472_6976; /* "virt" little-endian */
const DEV_BLK: u32 = 2;

fn read_u32(va: usize, off: usize) -> u32 {
    unsafe { core::ptr::read_volatile((va + off) as *const u32) }
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

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("virtioblk: probe start\n");
    let mut found_blk = false;
    let mut i = 0usize;
    while i < 8 {
        let slot = sys::mmio_virtio(i);
        if slot < 0 {
            i += 1;
            continue;
        }
        if sys::frame_map(slot as usize, sys::MMIO_VA, true) < 0 {
            let _ = sys::debug_write("virtioblk: map failed\n");
            i += 1;
            continue;
        }
        let magic = read_u32(sys::MMIO_VA, 0x000);
        let ver = read_u32(sys::MMIO_VA, 0x004);
        let did = read_u32(sys::MMIO_VA, 0x008);
        let _ = sys::frame_unmap(sys::MMIO_VA);

        if magic != MAGIC {
            i += 1;
            continue;
        }
        let _ = sys::debug_write("virtioblk: idx=");
        write_dec(i as u32);
        let _ = sys::debug_write(" ver=");
        write_dec(ver);
        let _ = sys::debug_write(" id=");
        write_dec(did);
        let _ = sys::debug_write("\n");
        if did == DEV_BLK {
            found_blk = true;
            let _ = sys::debug_write("virtioblk: found block device\n");
        }
        i += 1;
    }
    if found_blk {
        let _ = sys::debug_write("virtioblk: probe ok\n");
    } else {
        let _ = sys::debug_write("virtioblk: no block device\n");
    }
    /* Stay alive so smoke can see logs before exit path races. */
    loop {
        let _ = sys::yield_now();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("virtioblk: PANIC\n");
    sys::exit(1);
}
