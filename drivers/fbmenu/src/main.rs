//! fbmenu — simple ramfb menu + graphical terminal (1.15.1).
//!
//! Lives under `drivers/`. After `/fbdemo` exits, init spawns this server to
//! own the display: highlight menu, bounce demo, UART-echo terminal.
//! Smoke uses a timer auto-path (`SYS_DEBUG_READ` stays optional for humans).

#![no_std]
#![no_main]

mod font;

use deeproot_abi::{FB_BPP, FB_BYTES, FB_FOURCC_XR24, FB_HEIGHT, FB_PAGES, FB_STRIDE, FB_WIDTH};
use deeproot_user::sys;
use font::FONT8;

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

const BG: u32 = 0x0010_1828;
const FG: u32 = 0x00E8_E8E8;
const HI: u32 = 0x00E0_8040;
const DIM: u32 = 0x0060_7080;

const MENU_N: usize = 3;
const COLS: usize = 38;
const ROWS: usize = 20;

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
            let _ = sys::debug_write("fbmenu: dma timeout\n");
            return false;
        }
        core::hint::spin_loop();
    }
}

fn dma_transfer(
    mmio: usize,
    scratch_pa: usize,
    scratch_va: usize,
    control: u32,
    len: u32,
    buf_pa: u64,
) -> bool {
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
    wait_dma(scratch_va)
}

fn find_ramfb(mmio: usize, scratch_pa: usize, scratch_va: usize) -> Option<(u16, u32)> {
    let count_pa = scratch_pa + 64;
    let count_va = scratch_va + 64;
    let ctl = (FW_CFG_FILE_DIR << 16) | FW_CFG_DMA_CTL_SELECT | FW_CFG_DMA_CTL_READ;
    if !dma_transfer(mmio, scratch_pa, scratch_va, ctl, 4, count_pa as u64) {
        return None;
    }
    let n = u32::from_be(unsafe { core::ptr::read_volatile(count_va as *const u32) });
    if n == 0 || n > 64 {
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
            return Some((u16::from_be(f.select), u32::from_be(f.size)));
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

fn draw_char(x: u32, y: u32, ch: u8, fg: u32, bg: u32) {
    let g = if (32..=126).contains(&ch) {
        &FONT8[(ch - 32) as usize]
    } else {
        &FONT8[('?' as u8 - 32) as usize]
    };
    for row in 0..8u32 {
        let bits = g[row as usize];
        for col in 0..8u32 {
            let on = (bits >> (7 - col)) & 1 != 0;
            put_pixel(x + col, y + row, if on { fg } else { bg });
        }
    }
}

fn draw_text(x: u32, y: u32, s: &[u8], fg: u32, bg: u32) {
    let mut cx = x;
    for &b in s {
        draw_char(cx, y, b, fg, bg);
        cx = cx.saturating_add(8);
        if cx + 8 > FB_WIDTH {
            break;
        }
    }
}

fn hang(msg: &str) -> ! {
    let _ = sys::debug_write(msg);
    loop {
        let _ = sys::yield_now();
    }
}

fn setup_ramfb() -> bool {
    let _ = FB_BPP;
    let _ = FB_BYTES;

    let fw_slot = sys::mmio_fwcfg();
    if fw_slot < 0 {
        hang("fbmenu: fwcfg mint failed\n");
    }
    if sys::frame_map(fw_slot as usize, sys::FWCFG_MMIO_VA, true) < 0 {
        hang("fbmenu: fwcfg map failed\n");
    }

    let scratch_slot = sys::frame_alloc();
    if scratch_slot < 0 {
        hang("fbmenu: scratch alloc failed\n");
    }
    let scratch_pa = sys::frame_phys(scratch_slot as usize);
    if scratch_pa < 0 || sys::frame_map(scratch_slot as usize, sys::FWCFG_SCRATCH_VA, true) < 0 {
        hang("fbmenu: scratch map failed\n");
    }

    let fb_slot = sys::frame_alloc_n(FB_PAGES);
    if fb_slot < 0 {
        hang("fbmenu: fb alloc failed\n");
    }
    let fb_pa = sys::frame_phys(fb_slot as usize);
    if fb_pa < 0 || sys::frame_map(fb_slot as usize, sys::FB_VA, true) < 0 {
        hang("fbmenu: fb map failed\n");
    }

    let mmio = sys::FWCFG_MMIO_VA;
    let Some((select, fsz)) = find_ramfb(mmio, scratch_pa as usize, sys::FWCFG_SCRATCH_VA) else {
        hang("fbmenu: etc/ramfb missing\n");
    };
    if !configure_ramfb(
        mmio,
        scratch_pa as usize,
        sys::FWCFG_SCRATCH_VA,
        fb_pa as usize,
        select,
        fsz,
    ) {
        hang("fbmenu: ramfb cfg failed\n");
    }
    true
}

fn menu_label(i: usize) -> &'static [u8] {
    match i {
        0 => b"About DeepRoot",
        1 => b"Bounce demo",
        2 => b"Terminal",
        _ => b"?",
    }
}

fn draw_menu(sel: usize) {
    clear(BG);
    draw_text(16, 16, b"DeepRoot 1.15.1 fbmenu", FG, BG);
    draw_text(16, 32, b"w/s move  Enter select  q quit view", DIM, BG);
    for i in 0..MENU_N {
        let y = 64 + (i as u32) * 24;
        let (fg, bg) = if i == sel {
            fill_rect(12, y.saturating_sub(4), FB_WIDTH - 24, 20, HI);
            (BG, HI)
        } else {
            (FG, BG)
        };
        draw_text(24, y, menu_label(i), fg, bg);
    }
    draw_text(16, FB_HEIGHT - 24, b"./scripts/run-qemu.sh --gui", DIM, BG);
}

fn show_about() {
    clear(BG);
    draw_text(16, 24, b"DeepRoot microkernel", FG, BG);
    draw_text(16, 48, b"Capability RISC-V teaching OS", FG, BG);
    draw_text(16, 72, b"FB: QEMU ramfb via fw_cfg", FG, BG);
    draw_text(16, 96, b"Drivers live under drivers/", FG, BG);
    draw_text(16, 140, b"Auto-demo / press q", DIM, BG);
    let _ = sys::debug_write("fbmenu: select about\n");
}

fn run_bounce(frames: u32) {
    let _ = sys::debug_write("fbmenu: select bounce\n");
    let mut x: i32 = 40;
    let mut y: i32 = 40;
    let mut dx: i32 = 3;
    let mut dy: i32 = 2;
    let rw: i32 = 48;
    let rh: i32 = 32;
    let mut n = 0u32;
    while n < frames {
        clear(BG);
        draw_text(8, 8, b"Bounce  q=back", DIM, BG);
        fill_rect(x as u32, y as u32, rw as u32, rh as u32, HI);
        x += dx;
        y += dy;
        if x <= 0 || x + rw >= FB_WIDTH as i32 {
            dx = -dx;
            x += dx;
        }
        if y <= 16 || y + rh >= FB_HEIGHT as i32 {
            dy = -dy;
            y += dy;
        }
        if poll_quit() {
            return;
        }
        let _ = sys::sleep_ms(33);
        n += 1;
    }
}

/*
 * Terminal grid lives in .bss — keep it off the user stack. activate() used to
 * allocate ~800B frames (Term inline) and tripped illegal insn / faults on the
 * old 4-page spawn stacks when running beside the shell.
 */
struct Term {
    row: usize,
    col: usize,
}

static mut TERM_CELLS: [[u8; COLS]; ROWS] = [[b' '; COLS]; ROWS];

impl Term {
    fn reset() -> Self {
        unsafe {
            for r in 0..ROWS {
                for c in 0..COLS {
                    TERM_CELLS[r][c] = b' ';
                }
            }
        }
        Self { row: 0, col: 0 }
    }

    fn clear_row(&mut self, r: usize) {
        unsafe {
            for c in 0..COLS {
                TERM_CELLS[r][c] = b' ';
            }
        }
    }

    fn newline(&mut self) {
        self.col = 0;
        if self.row + 1 < ROWS {
            self.row += 1;
            self.clear_row(self.row);
        } else {
            unsafe {
                for r in 0..ROWS - 1 {
                    TERM_CELLS[r] = TERM_CELLS[r + 1];
                }
            }
            self.clear_row(ROWS - 1);
        }
    }

    fn put(&mut self, b: u8) {
        if b == b'\n' || b == b'\r' {
            self.newline();
            return;
        }
        if b == 0x08 || b == 0x7f {
            if self.col > 0 {
                self.col -= 1;
                unsafe {
                    TERM_CELLS[self.row][self.col] = b' ';
                }
            }
            return;
        }
        let ch = if (32..=126).contains(&b) { b } else { b'?' };
        unsafe {
            TERM_CELLS[self.row][self.col] = ch;
        }
        self.col += 1;
        if self.col >= COLS {
            self.newline();
        }
    }

    fn draw(&self) {
        clear(BG);
        draw_text(8, 4, b"Terminal  Esc/q back", DIM, BG);
        for r in 0..ROWS {
            let y = 20 + (r as u32) * 10;
            for c in 0..COLS {
                let x = 8 + (c as u32) * 8;
                let ch = unsafe { TERM_CELLS[r][c] };
                draw_char(x, y, ch, FG, BG);
            }
        }
    }
}

fn poll_byte() -> Option<u8> {
    let r = sys::debug_read_byte();
    if r >= 0 {
        Some(r as u8)
    } else {
        None
    }
}

fn poll_quit() -> bool {
    match poll_byte() {
        Some(b'q') | Some(b'Q') | Some(0x1b) => true,
        _ => false,
    }
}

fn run_terminal(auto_demo: bool) {
    let _ = sys::debug_write("fbmenu: terminal demo\n");
    let mut t = Term::reset();
    for &b in b"DeepRoot fb terminal\nType on serial; q/Esc exits.\n" {
        t.put(b);
    }
    t.draw();
    if auto_demo {
        for &b in b"hello from auto\n" {
            t.put(b);
            t.draw();
            let _ = sys::sleep_ms(20);
        }
        let _ = sys::sleep_ms(80);
        return;
    }
    loop {
        if let Some(b) = poll_byte() {
            if b == b'q' || b == b'Q' || b == 0x1b {
                return;
            }
            t.put(b);
            t.draw();
        } else {
            let _ = sys::sleep_ms(16);
        }
    }
}

fn activate(sel: usize) {
    match sel {
        0 => {
            show_about();
            while !poll_quit() {
                let _ = sys::sleep_ms(50);
            }
        }
        1 => run_bounce(10_000),
        2 => run_terminal(false),
        _ => {}
    }
}

/// Brief smoke walk so CI needles appear; then the menu waits for keys only.
fn smoke_markers() {
    show_about();
    let _ = sys::debug_write("fbmenu: select about\n");
    let _ = sys::sleep_ms(80);
    run_terminal(true);
    let _ = sys::sleep_ms(40);
}

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("fbmenu: start\n");
    if !setup_ramfb() {
        hang("fbmenu: setup failed\n");
    }
    let _ = sys::debug_write("fbmenu: ramfb ok\n");

    let mut sel: usize = 0;
    draw_menu(sel);
    let _ = sys::debug_write("fbmenu: menu ready\n");

    /* One-shot smoke markers, then human owns selection (no timer hijack). */
    smoke_markers();
    draw_menu(sel);
    let _ = sys::debug_write("fbmenu: your turn (w/s Enter, q in view)\n");

    /*
     * Interactive loop. Under `./scripts/run-qemu.sh --gui`, type on this
     * serial while fbmenu is foreground (`run fbmenu`). At the shell prompt
     * the shell consumes UART bytes — fbmenu will not see them.
     */
    loop {
        if let Some(b) = poll_byte() {
            match b {
                b'w' | b'W' | b'k' => {
                    sel = (sel + MENU_N - 1) % MENU_N;
                    draw_menu(sel);
                }
                b's' | b'S' | b'j' => {
                    sel = (sel + 1) % MENU_N;
                    draw_menu(sel);
                }
                b'\n' | b'\r' | b' ' => {
                    activate(sel);
                    draw_menu(sel);
                }
                _ => {}
            }
        } else {
            let _ = sys::sleep_ms(50);
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("fbmenu: PANIC\n");
    sys::exit(1);
}
