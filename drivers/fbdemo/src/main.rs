//! fbdemo — QEMU ramfb via fw_cfg + pixel draw (1.15.0).
//!
//! Configures `etc/ramfb`, maps a contiguous Frame buffer, then clear /
//! put_pixel / fill_rect. Serial markers for smoke under `-nographic`.
//! Exits after the draw demo so `/fbmenu` (1.15.1) can own the display.

#![no_std]
#![no_main]

use deeproot_abi::{FB_BPP, FB_BYTES, FB_FOURCC_XR24, FB_HEIGHT, FB_PAGES, FB_STRIDE, FB_WIDTH};
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

const FW_CFG_DMA_CTL_ERROR: u32 = 0x01;
const FW_CFG_DMA_CTL_READ: u32 = 0x02;
const FW_CFG_DMA_CTL_SELECT: u32 = 0x08;
const FW_CFG_DMA_CTL_WRITE: u32 = 0x10;
const FW_CFG_FILE_DIR: u32 = 0x19;

const FWCFG_DMA_OFF: usize = 0x10;

#[repr(C)]
struct FwCfgDmaAccess {
    control: u32,
    length: u32,
    address: u64,
}

#[repr(C)]
struct FwCfgFile {
    size: u32,
    select: u16,
    reserved: u16,
    name: [u8; 56],
}

#[repr(C)]
struct RamFbCfg {
    addr: u64,
    fourcc: u32,
    flags: u32,
    width: u32,
    height: u32,
    stride: u32,
}

fn wait_dma(acc_va: usize) -> bool {
    let mut spins = 0u32;
    loop {
        let ctl = unsafe { core::ptr::read_volatile(acc_va as *const u32) };
        let ctl = u32::from_be(ctl);
        if ctl & !FW_CFG_DMA_CTL_ERROR == 0 {
            return ctl & FW_CFG_DMA_CTL_ERROR == 0;
        }
        spins += 1;
        if spins > 10_000_000 {
            let _ = sys::debug_write("fbdemo: dma timeout\n");
            return false;
        }
        core::hint::spin_loop();
    }
}

fn dma_transfer(mmio: usize, scratch_pa: usize, scratch_va: usize, control: u32, len: u32, buf_pa: u64) -> bool {
    let acc = scratch_va as *mut FwCfgDmaAccess;
    unsafe {
        core::ptr::write_volatile(
            acc,
            FwCfgDmaAccess {
                control: control.to_be(),
                length: len.to_be(),
                address: buf_pa.to_be(),
            },
        );
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        let hi = ((scratch_pa as u64) >> 32) as u32;
        let lo = scratch_pa as u32;
        core::ptr::write_volatile((mmio + FWCFG_DMA_OFF) as *mut u32, hi.to_be());
        core::ptr::write_volatile((mmio + FWCFG_DMA_OFF + 4) as *mut u32, lo.to_be());
    }
    let ok = wait_dma(scratch_va);
    if !ok {
        let ctl = u32::from_be(unsafe { core::ptr::read_volatile(scratch_va as *const u32) });
        if ctl & FW_CFG_DMA_CTL_ERROR != 0 {
            let _ = sys::debug_write("fbdemo: dma error\n");
        }
    }
    ok
}

fn find_ramfb(mmio: usize, scratch_pa: usize, scratch_va: usize) -> Option<(u16, u32)> {
    let count_pa = scratch_pa + 64;
    let count_va = scratch_va + 64;
    let ctl = (FW_CFG_FILE_DIR << 16) | FW_CFG_DMA_CTL_SELECT | FW_CFG_DMA_CTL_READ;
    if !dma_transfer(mmio, scratch_pa, scratch_va, ctl, 4, count_pa as u64) {
        let _ = sys::debug_write("fbdemo: dir read fail\n");
        return None;
    }
    let n = u32::from_be(unsafe { core::ptr::read_volatile(count_va as *const u32) });
    if n == 0 || n > 64 {
        let _ = sys::debug_write("fbdemo: bad dir count\n");
        return None;
    }

    let ent_pa = scratch_pa + 128;
    let ent_va = scratch_va + 128;
    let target = b"etc/ramfb";
    for _ in 0..n {
        if !dma_transfer(
            mmio,
            scratch_pa,
            scratch_va,
            FW_CFG_DMA_CTL_READ,
            core::mem::size_of::<FwCfgFile>() as u32,
            ent_pa as u64,
        ) {
            return None;
        }
        let f = unsafe { &*(ent_va as *const FwCfgFile) };
        let mut match_ok = true;
        for (i, b) in target.iter().enumerate() {
            if f.name[i] != *b {
                match_ok = false;
                break;
            }
        }
        if match_ok && f.name[target.len()] == 0 {
            let sel = u16::from_be(f.select);
            let sz = u32::from_be(f.size);
            return Some((sel, sz));
        }
    }
    None
}

fn configure_ramfb(
    mmio: usize,
    scratch_pa: usize,
    scratch_va: usize,
    fb_pa: usize,
    select: u16,
    file_size: u32,
) -> bool {
    let cfg_pa = scratch_pa + 256;
    let cfg_va = scratch_va + 256;
    let cfg = RamFbCfg {
        addr: (fb_pa as u64).to_be(),
        fourcc: FB_FOURCC_XR24.to_be(),
        flags: 0u32.to_be(),
        width: FB_WIDTH.to_be(),
        height: FB_HEIGHT.to_be(),
        stride: FB_STRIDE.to_be(),
    };
    unsafe {
        core::ptr::write_volatile(cfg_va as *mut RamFbCfg, cfg);
    }
    let len = if file_size == 0 {
        core::mem::size_of::<RamFbCfg>() as u32
    } else {
        file_size
    };
    let ctl = ((select as u32) << 16) | FW_CFG_DMA_CTL_SELECT | FW_CFG_DMA_CTL_WRITE;
    dma_transfer(mmio, scratch_pa, scratch_va, ctl, len, cfg_pa as u64)
}

fn fb_ptr() -> *mut u32 {
    sys::FB_VA as *mut u32
}

fn clear(color: u32) {
    let n = (FB_WIDTH * FB_HEIGHT) as usize;
    let p = fb_ptr();
    unsafe {
        for i in 0..n {
            core::ptr::write_volatile(p.add(i), color);
        }
    }
}

fn put_pixel(x: u32, y: u32, color: u32) {
    if x >= FB_WIDTH || y >= FB_HEIGHT {
        return;
    }
    let off = (y * FB_WIDTH + x) as usize;
    unsafe {
        core::ptr::write_volatile(fb_ptr().add(off), color);
    }
}

fn fill_rect(x0: u32, y0: u32, w: u32, h: u32, color: u32) {
    let x1 = (x0 + w).min(FB_WIDTH);
    let y1 = (y0 + h).min(FB_HEIGHT);
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            put_pixel(x, y, color);
            x += 1;
        }
        y += 1;
    }
}

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("fbdemo: start\n");
    let _ = FB_BPP;
    let _ = FB_BYTES;

    let fw_slot = sys::mmio_fwcfg();
    if fw_slot < 0 {
        let _ = sys::debug_write("fbdemo: fwcfg mint failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    if sys::frame_map(fw_slot as usize, sys::FWCFG_MMIO_VA, true) < 0 {
        let _ = sys::debug_write("fbdemo: fwcfg map failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }

    let scratch_slot = sys::frame_alloc();
    if scratch_slot < 0 {
        let _ = sys::debug_write("fbdemo: scratch alloc failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    let scratch_pa = sys::frame_phys(scratch_slot as usize);
    if scratch_pa < 0
        || sys::frame_map(scratch_slot as usize, sys::FWCFG_SCRATCH_VA, true) < 0
    {
        let _ = sys::debug_write("fbdemo: scratch map failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }

    let fb_slot = sys::frame_alloc_n(FB_PAGES);
    if fb_slot < 0 {
        let _ = sys::debug_write("fbdemo: fb alloc failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    let fb_pa = sys::frame_phys(fb_slot as usize);
    if fb_pa < 0 || sys::frame_map(fb_slot as usize, sys::FB_VA, true) < 0 {
        let _ = sys::debug_write("fbdemo: fb map failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }

    let mmio = sys::FWCFG_MMIO_VA;
    let Some((select, fsz)) = find_ramfb(mmio, scratch_pa as usize, sys::FWCFG_SCRATCH_VA) else {
        let _ = sys::debug_write("fbdemo: etc/ramfb missing\n");
        loop {
            let _ = sys::yield_now();
        }
    };
    let _ = sys::debug_write("fbdemo: found etc/ramfb\n");
    if !configure_ramfb(
        mmio,
        scratch_pa as usize,
        sys::FWCFG_SCRATCH_VA,
        fb_pa as usize,
        select,
        fsz,
    ) {
        let _ = sys::debug_write("fbdemo: ramfb cfg failed\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    let _ = sys::debug_write("fbdemo: ramfb ok\n");

    /* Dark blue clear. */
    clear(0x0010_2040);
    /* Spot-check a corner pixel. */
    let sample = unsafe { core::ptr::read_volatile(fb_ptr()) };
    if sample != 0x0010_2040 {
        let _ = sys::debug_write("fbdemo: clear mismatch\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    let _ = sys::debug_write("fbdemo: clear ok\n");

    fill_rect(40, 40, 120, 80, 0x00E0_8040);
    put_pixel(0, 0, 0x00FF_0000);
    put_pixel(FB_WIDTH - 1, FB_HEIGHT - 1, 0x0000_FF00);
    let mid = unsafe {
        core::ptr::read_volatile(fb_ptr().add((60 * FB_WIDTH + 60) as usize))
    };
    if mid != 0x00E0_8040 {
        let _ = sys::debug_write("fbdemo: fill_rect mismatch\n");
        loop {
            let _ = sys::yield_now();
        }
    }
    let _ = sys::debug_write("fbdemo: fill_rect ok\n");
    /* GUI refresh / menu ownership continues in /fbmenu (1.15.1). */
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("fbdemo: PANIC\n");
    sys::exit(1);
}
