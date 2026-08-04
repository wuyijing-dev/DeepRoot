//! badapple — realtime ASCII from BA02 4-bit frames.
//!
//! Decode → map 16 gray levels to an ASCII ramp → one bulk console write,
//! paced with SYS_TIME. Press `q` to quit.

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
const _: [(); 1] = [(); (FRAMES.len() > 10_000) as usize];

const MAX_PIXELS: usize = 128 * 48;
const MAX_PACKED: usize = (MAX_PIXELS + 1) / 2;
/* ANSI home + w chars + newline per row */
const MAX_FRAME_CHARS: usize = 8 + MAX_PIXELS + 128;

/// 16-level ramp: black → white (dense → blank).
const RAMP: &[u8; 16] = b"@&#%*+=:;~-_,.  ";

struct Header {
    width: usize,
    height: usize,
    fps: usize,
    bits: usize,
    nframes: usize,
    body: &'static [u8],
}

fn parse_header(blob: &'static [u8]) -> Option<Header> {
    if blob.len() < 12 {
        return None;
    }
    let ver = &blob[0..4];
    if ver != b"BA02" && ver != b"BA01" {
        return None;
    }
    let width = blob[4] as usize;
    let height = blob[5] as usize;
    let fps = blob[6] as usize;
    let bits = if ver == b"BA02" {
        blob[7] as usize
    } else {
        1
    };
    let nframes = u32::from_le_bytes([blob[8], blob[9], blob[10], blob[11]]) as usize;
    if width == 0 || height == 0 || fps == 0 || nframes == 0 {
        return None;
    }
    if !(bits == 1 || bits == 4) || width * height > MAX_PIXELS {
        return None;
    }
    Some(Header {
        width,
        height,
        fps,
        bits,
        nframes,
        body: &blob[12..],
    })
}

fn packed_len(w: usize, h: usize, bits: usize) -> usize {
    if bits == 4 {
        (w * h + 1) / 2
    } else {
        (w * h + 7) / 8
    }
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

fn level_at(pack: &[u8], i: usize, bits: usize) -> u8 {
    if bits == 4 {
        let b = pack[i >> 1];
        if (i & 1) == 0 {
            b >> 4
        } else {
            b & 0x0f
        }
    } else if (pack[i >> 3] & (1 << (7 - (i & 7)))) != 0 {
        15
    } else {
        0
    }
}

/*
 * render_frame - build one ANSI+ASCII frame and write it in a single syscall
 */
fn render_frame(pack: &[u8], w: usize, h: usize, bits: usize, out: &mut [u8]) -> usize {
    let mut n = 0usize;
    /* Cursor home (no full clear — cheaper, less flicker). */
    out[n] = 0x1b;
    out[n + 1] = b'[';
    out[n + 2] = b'H';
    n += 3;
    for y in 0..h {
        for x in 0..w {
            let lv = level_at(pack, y * w + x, bits) as usize;
            out[n] = RAMP[lv & 15];
            n += 1;
        }
        out[n] = b'\n';
        n += 1;
    }
    let _ = sys::debug_write(core::str::from_utf8(&out[..n]).unwrap_or(""));
    n
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
        /* Short busy-poll then yield — keeps pacing tight under TCG. */
        let _ = sys::yield_now();
    }
}

#[no_mangle]
pub extern "C" fn main() {
    let hdr = match parse_header(FRAMES) {
        Some(h) => h,
        None => {
            let _ = sys::debug_write("badapple: bad frames blob\n");
            sys::exit(1);
        }
    };

    let _ = sys::debug_write("badapple: realtime ASCII 16-level (q=quit)\n");
    let _ = sys::debug_write("\x1b[2J\x1b[H");

    let plen = packed_len(hdr.width, hdr.height, hdr.bits);
    let mut prev = [0u8; MAX_PACKED];
    let mut cur = [0u8; MAX_PACKED];
    let mut xor_buf = [0u8; MAX_PACKED];
    let mut frame = [0u8; MAX_FRAME_CHARS];

    let period_ms = 1000u64 / hdr.fps as u64;
    let mut off = 0usize;
    let mut deadline = sys::time_ms();

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
        if !rle_decode(enc, &mut xor_buf[..plen]) {
            let _ = sys::debug_write("badapple: rle error\n");
            break;
        }
        for i in 0..plen {
            cur[i] = prev[i] ^ xor_buf[i];
        }
        render_frame(&cur[..plen], hdr.width, hdr.height, hdr.bits, &mut frame);
        prev[..plen].copy_from_slice(&cur[..plen]);

        deadline = deadline.wrapping_add(period_ms);
        let now = sys::time_ms();
        if now + 2 < deadline {
            wait_until(deadline);
        } else if now > deadline.wrapping_add(period_ms) {
            /* Severely behind: drop catch-up waits. */
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
