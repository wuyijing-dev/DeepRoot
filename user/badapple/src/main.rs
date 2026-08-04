//! badapple — realtime ASCII render from compressed BA01 frames.
//!
//! Frames are 1-bit pixel buffers (xor+RLE). Each tick we decode → map to
//! ASCII → draw, then pace with SYS_TIME so playback tracks the encoded fps.
//! Press `q` to quit early.

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

static FRAMES: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/frames.ba01"));

/* Fail the build if the compressed stream is missing / empty. */
const _: [(); 1] = [(); (FRAMES.len() > 10_000) as usize];

const MAX_PIXELS: usize = 64 * 32;
const MAX_PACKED: usize = (MAX_PIXELS + 7) / 8;

struct Header {
    width: usize,
    height: usize,
    fps: usize,
    nframes: usize,
    body: &'static [u8],
}

fn parse_header(blob: &'static [u8]) -> Option<Header> {
    if blob.len() < 12 || &blob[0..4] != b"BA01" {
        return None;
    }
    let width = blob[4] as usize;
    let height = blob[5] as usize;
    let fps = blob[6] as usize;
    /* blob[7] = pad; nframes at 8..12 */
    let nframes = u32::from_le_bytes([blob[8], blob[9], blob[10], blob[11]]) as usize;
    if width == 0 || height == 0 || fps == 0 || nframes == 0 || width * height > MAX_PIXELS {
        return None;
    }
    Some(Header {
        width,
        height,
        fps,
        nframes,
        body: &blob[12..],
    })
}

fn rle_decode(src: &[u8], dst: &mut [u8]) -> bool {
    let mut si = 0usize;
    let mut di = 0usize;
    while si + 1 < src.len() && di < dst.len() {
        let count = src[si] as usize;
        let val = src[si + 1];
        si += 2;
        if count == 0 || di + count > dst.len() {
            return false;
        }
        for _ in 0..count {
            dst[di] = val;
            di += 1;
        }
    }
    di == dst.len()
}

fn pixel(bits: &[u8], i: usize) -> bool {
    (bits[i >> 3] & (1 << (7 - (i & 7)))) != 0
}

/*
 * render_frame - map 1-bit pixels to ASCII and redraw the terminal region
 *
 * Realtime path: no pre-baked character frames; glyphs are chosen here.
 */
fn render_frame(bits: &[u8], w: usize, h: usize, line: &mut [u8]) {
    let _ = sys::debug_write("\x1b[H");
    for y in 0..h {
        let mut n = 0usize;
        for x in 0..w {
            line[n] = if pixel(bits, y * w + x) { b'#' } else { b' ' };
            n += 1;
        }
        line[n] = b'\n';
        n += 1;
        let _ = sys::debug_write(core::str::from_utf8(&line[..n]).unwrap_or("\n"));
    }
}

fn quit_requested() -> bool {
    let c = sys::debug_read_byte();
    if c < 0 {
        return false;
    }
    c == b'q' as isize || c == b'Q' as isize || c == 27
}

fn wait_until(deadline_ms: u64) {
    while sys::time_ms() < deadline_ms {
        if quit_requested() {
            return;
        }
        let _ = sys::yield_now();
    }
}

#[no_mangle]
pub extern "C" fn main() {
    let hdr = match parse_header(FRAMES) {
        Some(h) => h,
        None => {
            let _ = sys::debug_write("badapple: bad frames.ba01\n");
            sys::exit(1);
        }
    };

    let _ = sys::debug_write("badapple: realtime ASCII (q=quit)\n");
    let _ = sys::debug_write("\x1b[2J\x1b[H");

    let packed_len = (hdr.width * hdr.height + 7) / 8;
    let mut prev = [0u8; MAX_PACKED];
    let mut cur = [0u8; MAX_PACKED];
    let mut xor_buf = [0u8; MAX_PACKED];
    let mut line = [0u8; 128];

    let period_ms = 1000u64 / hdr.fps as u64;
    let mut off = 0usize;
    let start = sys::time_ms();
    let mut deadline = start;

    for _fi in 0..hdr.nframes {
        if quit_requested() {
            break;
        }
        if off + 2 > hdr.body.len() {
            let _ = sys::debug_write("badapple: truncated stream\n");
            break;
        }
        let enc_len = u16::from_le_bytes([hdr.body[off], hdr.body[off + 1]]) as usize;
        off += 2;
        if off + enc_len > hdr.body.len() || enc_len > xor_buf.len() {
            let _ = sys::debug_write("badapple: bad frame\n");
            break;
        }
        let enc = &hdr.body[off..off + enc_len];
        off += enc_len;
        if !rle_decode(enc, &mut xor_buf[..packed_len]) {
            let _ = sys::debug_write("badapple: rle error\n");
            break;
        }
        for i in 0..packed_len {
            cur[i] = prev[i] ^ xor_buf[i];
        }
        render_frame(&cur[..packed_len], hdr.width, hdr.height, &mut line);
        prev[..packed_len].copy_from_slice(&cur[..packed_len]);

        deadline = deadline.wrapping_add(period_ms);
        /* If we fell behind (serial slow), skip waiting rather than snowball. */
        let now = sys::time_ms();
        if now + 5 < deadline {
            wait_until(deadline);
        } else {
            deadline = now;
        }
    }

    let _ = sys::debug_write("\nbadapple: done\n");
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("badapple: PANIC\n");
    sys::exit(1);
}
