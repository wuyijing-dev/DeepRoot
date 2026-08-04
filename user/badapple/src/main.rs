//! badapple — realtime ASCII from BA02 4-bit frames @ source 30fps.
//!
//! Anti-flicker: hide cursor, optional sync dump, home+EL per row (no scroll),
//! status line via CUP once per second. Wall-clock pacing with draw-skips.

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

const MAX_PIXELS: usize = 80 * 24;
const MAX_PACKED: usize = (MAX_PIXELS + 1) / 2;
const MAX_FRAME_CHARS: usize = 64 + MAX_PIXELS + 256;

const RAMP: &[u8; 16] = b"@&#%*+=:;~-_,.  ";

static mut PREV: [u8; MAX_PACKED] = [0; MAX_PACKED];
static mut CUR: [u8; MAX_PACKED] = [0; MAX_PACKED];
static mut XOR_BUF: [u8; MAX_PACKED] = [0; MAX_PACKED];
static mut FRAME: [u8; MAX_FRAME_CHARS] = [0; MAX_FRAME_CHARS];

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

fn push_u32(out: &mut [u8], mut n: usize, mut i: usize) -> usize {
    if n == 0 {
        out[i] = b'0';
        return i + 1;
    }
    let mut tmp = [0u8; 20];
    let mut t = 0usize;
    while n > 0 {
        tmp[t] = b'0' + (n % 10) as u8;
        n /= 10;
        t += 1;
    }
    while t > 0 {
        t -= 1;
        out[i] = tmp[t];
        i += 1;
    }
    i
}

fn push_str(out: &mut [u8], i: usize, s: &[u8]) -> usize {
    out[i..i + s.len()].copy_from_slice(s);
    i + s.len()
}

fn write_banner(hdr: &Header) {
    let mut b = [0u8; 192];
    let mut i = 0usize;
    i = push_str(&mut b, i, b"badapple: ");
    i = push_u32(&mut b, hdr.width, i);
    i = push_str(&mut b, i, b"x");
    i = push_u32(&mut b, hdr.height, i);
    i = push_str(&mut b, i, b"  enc_fps=");
    i = push_u32(&mut b, hdr.fps, i);
    i = push_str(&mut b, i, b"  frames=");
    i = push_u32(&mut b, hdr.nframes, i);
    i = push_str(&mut b, i, b"  dur_s=");
    i = push_u32(&mut b, hdr.nframes / hdr.fps.max(1), i);
    i = push_str(&mut b, i, b" (30fps timeline; q=quit)\n");
    let _ = sys::debug_write(core::str::from_utf8(&b[..i]).unwrap_or("\n"));
}

/*
 * render_frame - tear-resistant redraw
 *
 * - CSI ?2026 : buffered sync (ignored by dumb terminals)
 * - cursor home, each row ends with EL (erase to EOL) so leftovers never flash
 * - no extra blank lines that would scroll the viewport
 * - status via CUP under the picture (updated by caller rate)
 */
fn render_frame(
    pack: &[u8],
    w: usize,
    h: usize,
    bits: usize,
    fi: usize,
    nframes: usize,
    enc_fps: usize,
    drawn: usize,
    show_status: bool,
    out: &mut [u8],
) {
    let mut n = 0usize;
    /* Begin synchronized update + home. */
    n = push_str(out, n, b"\x1b[?2026h\x1b[H");
    for y in 0..h {
        for x in 0..w {
            let lv = level_at(pack, y * w + x, bits) as usize;
            out[n] = RAMP[lv & 15];
            n += 1;
        }
        /* Erase to end of line, then next line — avoids scroll flicker. */
        n = push_str(out, n, b"\x1b[K\n");
    }
    if show_status {
        /* Row h+2, column 1. */
        n = push_str(out, n, b"\x1b[");
        n = push_u32(out, h + 2, n);
        n = push_str(out, n, b";1Hframe ");
        n = push_u32(out, fi + 1, n);
        n = push_str(out, n, b"/");
        n = push_u32(out, nframes, n);
        n = push_str(out, n, b"  fps=");
        n = push_u32(out, enc_fps, n);
        n = push_str(out, n, b"  drawn=");
        n = push_u32(out, drawn, n);
        n = push_str(out, n, b"\x1b[K");
    }
    n = push_str(out, n, b"\x1b[?2026l");
    let _ = sys::debug_write(core::str::from_utf8(&out[..n]).unwrap_or(""));
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

fn decode_one(
    body: &[u8],
    off: &mut usize,
    plen: usize,
    xor_buf: &mut [u8],
    prev: &mut [u8],
    cur: &mut [u8],
) -> bool {
    if *off + 2 > body.len() {
        return false;
    }
    let enc_len = u16::from_le_bytes([body[*off], body[*off + 1]]) as usize;
    *off += 2;
    if *off + enc_len > body.len() || enc_len > xor_buf.len() {
        return false;
    }
    let enc = &body[*off..*off + enc_len];
    *off += enc_len;
    if !rle_decode(enc, &mut xor_buf[..plen]) {
        return false;
    }
    for i in 0..plen {
        cur[i] = prev[i] ^ xor_buf[i];
    }
    prev[..plen].copy_from_slice(&cur[..plen]);
    true
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

    write_banner(&hdr);
    /* Alternate screen + hide cursor. */
    let _ = sys::debug_write("\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H");
    write_banner(&hdr);

    let plen = packed_len(hdr.width, hdr.height, hdr.bits);
    let (prev, cur, xor_buf, frame) = unsafe {
        (
            &mut *core::ptr::addr_of_mut!(PREV),
            &mut *core::ptr::addr_of_mut!(CUR),
            &mut *core::ptr::addr_of_mut!(XOR_BUF),
            &mut *core::ptr::addr_of_mut!(FRAME),
        )
    };
    prev[..plen].fill(0);

    let fps = hdr.fps as u64;
    let mut off = 0usize;
    let t0 = sys::time_ms();
    let mut drawn = 0usize;
    let status_every = if hdr.fps >= 15 { hdr.fps } else { 1 };

    for fi in 0..hdr.nframes {
        if quit_requested() {
            break;
        }
        if !decode_one(hdr.body, &mut off, plen, xor_buf, prev, cur) {
            let _ = sys::debug_write("badapple: decode error\n");
            break;
        }

        let elapsed = sys::time_ms().wrapping_sub(t0);
        let target = ((elapsed * fps) / 1000) as usize;
        if fi < target {
            continue;
        }

        drawn += 1;
        let show_status = drawn == 1 || drawn % status_every == 0 || fi + 1 == hdr.nframes;
        render_frame(
            &cur[..plen],
            hdr.width,
            hdr.height,
            hdr.bits,
            fi,
            hdr.nframes,
            hdr.fps,
            drawn,
            show_status,
            frame,
        );

        let next_ms = t0.wrapping_add(((fi as u64 + 1) * 1000) / fps);
        wait_until(next_ms);
    }

    /* Restore terminal. */
    let _ = sys::debug_write("\x1b[?25h\x1b[?1049l");

    let elapsed = sys::time_ms().wrapping_sub(t0);
    let mut b = [0u8; 96];
    let mut i = 0usize;
    i = push_str(&mut b, i, b"badapple: done  wall_ms=");
    i = push_u32(&mut b, elapsed as usize, i);
    i = push_str(&mut b, i, b"  drawn=");
    i = push_u32(&mut b, drawn, i);
    i = push_str(&mut b, i, b"/");
    i = push_u32(&mut b, hdr.nframes, i);
    i = push_str(&mut b, i, b"\n");
    let _ = sys::debug_write(core::str::from_utf8(&b[..i]).unwrap_or("\n"));
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("\x1b[?25h\x1b[?1049lbadapple: PANIC\n");
    sys::exit(1);
}
