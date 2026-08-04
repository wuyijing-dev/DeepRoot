//! Teaching block layer — DRFS on virtio-blk (1.6) or ramdisk fallback.
//!
//! On-disk layout (DeepRoot teaching FS image, magic `DRFS`):
//!
//! ```text
//! [0..4)    magic b"DRFS"
//! [4..8)    u32 little-endian version (=1)
//! [8..12)   u32 little-endian file count
//! [12..16)  reserved
//! [16..)    directory: N × 48-byte entries
//!             name[32]  NUL-padded ASCII
//!             offset    u32 LE  (byte offset into image)
//!             length    u32 LE
//!             flags     u32 LE  (bit0 = text)
//! then file payloads at the recorded offsets
//! ```
//!
//! Byte read/write go through [`virtio_blk`] when present; otherwise a static
//! ramdisk. Shell `ls` / `cat` stay on this DRFS API.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::println;
use crate::virtio_blk;

pub const DRFS_BYTES: usize = 64 * 1024;
const MAGIC: &[u8; 4] = b"DRFS";
const VERSION: u32 = 1;
const DIR_OFF: usize = 16;
const ENTRY_SIZE: usize = 48;
const NAME_LEN: usize = 32;
const MAX_FILES: usize = 8;

static mut RAMDISK: [u8; DRFS_BYTES] = [0; DRFS_BYTES];
static READY: AtomicBool = AtomicBool::new(false);
static USE_VIRTIO: AtomicBool = AtomicBool::new(false);
static STORE_SIZE: AtomicUsize = AtomicUsize::new(DRFS_BYTES);

#[derive(Clone, Copy)]
pub struct DirEnt {
    pub name: [u8; NAME_LEN],
    pub name_len: usize,
    pub offset: u32,
    pub length: u32,
    pub flags: u32,
}

impl DirEnt {
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    pub fn is_text(&self) -> bool {
        self.flags & 1 != 0
    }
}

fn backend_read(off: usize, out: &mut [u8]) -> usize {
    if USE_VIRTIO.load(Ordering::Relaxed) {
        virtio_blk::read_bytes(off, out)
    } else {
        let size = STORE_SIZE.load(Ordering::Relaxed);
        if off >= size {
            return 0;
        }
        let n = out.len().min(size - off);
        unsafe {
            out[..n].copy_from_slice(&RAMDISK[off..off + n]);
        }
        n
    }
}

fn backend_write(off: usize, data: &[u8]) -> usize {
    if USE_VIRTIO.load(Ordering::Relaxed) {
        virtio_blk::write_bytes(off, data)
    } else {
        let size = STORE_SIZE.load(Ordering::Relaxed);
        if off >= size {
            return 0;
        }
        let n = data.len().min(size - off);
        unsafe {
            RAMDISK[off..off + n].copy_from_slice(&data[..n]);
        }
        n
    }
}

fn write_u32(off: usize, v: u32) {
    let b = v.to_le_bytes();
    let _ = backend_write(off, &b);
}

fn read_u32(off: usize) -> u32 {
    let mut b = [0u8; 4];
    let n = backend_read(off, &mut b);
    if n < 4 {
        return 0;
    }
    u32::from_le_bytes(b)
}

fn put_bytes(off: usize, data: &[u8]) {
    let _ = backend_write(off, data);
}

fn write_entry(slot: usize, name: &str, offset: u32, length: u32, flags: u32) {
    let base = DIR_OFF + slot * ENTRY_SIZE;
    let mut name_buf = [0u8; NAME_LEN];
    let n = name.len().min(NAME_LEN - 1);
    name_buf[..n].copy_from_slice(&name.as_bytes()[..n]);
    put_bytes(base, &name_buf);
    write_u32(base + NAME_LEN, offset);
    write_u32(base + NAME_LEN + 4, length);
    write_u32(base + NAME_LEN + 8, flags);
}

fn has_drfs_magic() -> bool {
    let mut magic = [0u8; 4];
    if backend_read(0, &mut magic) < 4 {
        return false;
    }
    &magic == MAGIC
}

fn format_drfs() {
    /* Zero the teaching image window. */
    let zero = [0u8; 512];
    let mut off = 0usize;
    while off < DRFS_BYTES {
        let n = (DRFS_BYTES - off).min(zero.len());
        let _ = backend_write(off, &zero[..n]);
        off += n;
    }

    let data_base = DIR_OFF + MAX_FILES * ENTRY_SIZE;
    let files: &[(&str, &[u8])] = &[
        (
            "block.txt",
            b"DeepRoot 1.6.0 - this file lives on the block device (DRFS via virtio-blk or ramdisk).\n",
        ),
        (
            "from-block",
            b"cat from-block  # path served via block::read\n",
        ),
        ("blk-version", b"1.6.0\n"),
    ];

    put_bytes(0, MAGIC);
    write_u32(4, VERSION);
    write_u32(8, files.len() as u32);
    write_u32(12, 0);

    let mut cursor = data_base;
    for (i, (name, data)) in files.iter().enumerate() {
        put_bytes(cursor, data);
        write_entry(i, name, cursor as u32, data.len() as u32, 1);
        cursor += data.len();
        cursor = (cursor + 3) & !3;
    }
}

/*
 * init - prefer virtio-blk from FDT; else ramdisk. Ensure DRFS image exists.
 */
pub fn init() {
    let virt = virtio_blk::init();
    if virt {
        USE_VIRTIO.store(true, Ordering::Relaxed);
        let cap = virtio_blk::capacity_bytes();
        let window = if cap == 0 {
            DRFS_BYTES
        } else {
            core::cmp::min(cap, DRFS_BYTES)
        };
        STORE_SIZE.store(window, Ordering::Relaxed);
    } else {
        USE_VIRTIO.store(false, Ordering::Relaxed);
        STORE_SIZE.store(DRFS_BYTES, Ordering::Relaxed);
        unsafe {
            RAMDISK.fill(0);
        }
    }

    if !has_drfs_magic() {
        format_drfs();
        println!("block: formatted DRFS image ({} bytes window)", size());
    } else {
        println!("block: found existing DRFS image");
    }

    READY.store(true, Ordering::Relaxed);
    let backend = if USE_VIRTIO.load(Ordering::Relaxed) {
        "virtio-blk"
    } else {
        "ramdisk"
    };
    println!(
        "block: {} ready size={} files={} (DRFS)",
        backend,
        size(),
        file_count()
    );
}

pub fn ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub fn size() -> usize {
    STORE_SIZE.load(Ordering::Relaxed)
}

pub fn using_virtio() -> bool {
    USE_VIRTIO.load(Ordering::Relaxed)
}

pub fn read(off: usize, out: &mut [u8]) -> usize {
    if !ready() {
        return 0;
    }
    let lim = STORE_SIZE.load(Ordering::Relaxed);
    if off >= lim {
        return 0;
    }
    let n = out.len().min(lim - off);
    backend_read(off, &mut out[..n])
}

pub fn write(off: usize, data: &[u8]) -> usize {
    if !ready() {
        return 0;
    }
    let lim = STORE_SIZE.load(Ordering::Relaxed);
    if off >= lim {
        return 0;
    }
    let n = data.len().min(lim - off);
    backend_write(off, &data[..n])
}

pub fn file_count() -> usize {
    if !ready() || !has_drfs_magic() {
        return 0;
    }
    read_u32(8) as usize
}

pub fn dirent(idx: usize) -> Option<DirEnt> {
    let n = file_count();
    if idx >= n || idx >= MAX_FILES {
        return None;
    }
    let base = DIR_OFF + idx * ENTRY_SIZE;
    let mut name = [0u8; NAME_LEN];
    if backend_read(base, &mut name) < NAME_LEN {
        return None;
    }
    let name_len = name.iter().position(|&c| c == 0).unwrap_or(NAME_LEN);
    Some(DirEnt {
        name,
        name_len,
        offset: read_u32(base + NAME_LEN),
        length: read_u32(base + NAME_LEN + 4),
        flags: read_u32(base + NAME_LEN + 8),
    })
}

pub fn lookup(name: &str, out: &mut [u8]) -> Option<(usize, usize, bool)> {
    let n = file_count();
    for i in 0..n {
        let Some(ent) = dirent(i) else {
            continue;
        };
        if ent.name_str() == name {
            let total = ent.length as usize;
            let mut nread = 0usize;
            if !out.is_empty() && total > 0 {
                let cap = out.len().min(total);
                nread = read(ent.offset as usize, &mut out[..cap]);
            }
            return Some((nread, total, ent.is_text()));
        }
    }
    None
}
