//! Teaching block device (1.4) — RAM disk stand-in for virtio-blk.
//!
//! Layout (DeepRoot teaching FS image, magic `DRFS`):
//!
//! ```text
//! [0..4)    magic b"DRFS"
//! [4..8)    u32 little-endian version (=1)
//! [8..12)   u32 little-endian file count
//! [12..16)  reserved
//! [16..)    directory: N × 48-byte entries
//!             name[32]  NUL-padded ASCII
//!             offset    u32 LE  (byte offset into DISK)
//!             length    u32 LE
//!             flags     u32 LE  (bit0 = text)
//! then file payloads at the recorded offsets
//! ```
//!
//! QEMU virtio-blk can replace the DISK[] backend later without changing
//! the DRFS layout or the shell path API (`ls` / `cat`).

use core::sync::atomic::{AtomicBool, Ordering};

use crate::println;

pub const DISK_BYTES: usize = 16 * 1024;
const MAGIC: &[u8; 4] = b"DRFS";
const VERSION: u32 = 1;
const DIR_OFF: usize = 16;
const ENTRY_SIZE: usize = 48;
const NAME_LEN: usize = 32;
const MAX_FILES: usize = 8;

static mut DISK: [u8; DISK_BYTES] = [0; DISK_BYTES];
static READY: AtomicBool = AtomicBool::new(false);

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

fn write_u32(off: usize, v: u32) {
    let b = v.to_le_bytes();
    unsafe {
        DISK[off..off + 4].copy_from_slice(&b);
    }
}

fn read_u32(off: usize) -> u32 {
    let mut b = [0u8; 4];
    unsafe {
        b.copy_from_slice(&DISK[off..off + 4]);
    }
    u32::from_le_bytes(b)
}

fn put_bytes(off: usize, data: &[u8]) {
    unsafe {
        DISK[off..off + data.len()].copy_from_slice(data);
    }
}

fn write_entry(slot: usize, name: &str, offset: u32, length: u32, flags: u32) {
    let base = DIR_OFF + slot * ENTRY_SIZE;
    let mut name_buf = [0u8; NAME_LEN];
    let n = name.len().min(NAME_LEN - 1);
    name_buf[..n].copy_from_slice(&name.as_bytes()[..n]);
    unsafe {
        DISK[base..base + NAME_LEN].copy_from_slice(&name_buf);
    }
    write_u32(base + NAME_LEN, offset);
    write_u32(base + NAME_LEN + 4, length);
    write_u32(base + NAME_LEN + 8, flags);
}

/*
 * init - format a DRFS image on the ramdisk and seed teaching text files
 */
pub fn init() {
    unsafe {
        DISK.fill(0);
    }

    /* File payloads start after the directory table. */
    let data_base = DIR_OFF + MAX_FILES * ENTRY_SIZE;
    let files: &[(&str, &[u8])] = &[
        (
            "block.txt",
            b"DeepRoot 1.4.1 - this file lives on the block device (DRFS), not in embed ramfs.\n",
        ),
        (
            "from-block",
            b"cat from-block  # path served via block::read\n",
        ),
        ("blk-version", b"1.4.1\n"),
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
        /* Keep payloads 4-byte aligned for neatness. */
        cursor = (cursor + 3) & !3;
    }

    READY.store(true, Ordering::Relaxed);
    println!(
        "block: ramdisk ready size={} files={} (DRFS / virtio-blk stand-in)",
        DISK_BYTES,
        files.len()
    );
}

pub fn ready() -> bool {
    READY.load(Ordering::Relaxed)
}

pub fn size() -> usize {
    DISK_BYTES
}

pub fn read(off: usize, out: &mut [u8]) -> usize {
    if !ready() || off >= DISK_BYTES {
        return 0;
    }
    let n = out.len().min(DISK_BYTES - off);
    unsafe {
        out[..n].copy_from_slice(&DISK[off..off + n]);
    }
    n
}

pub fn write(off: usize, data: &[u8]) -> usize {
    if !ready() || off >= DISK_BYTES {
        return 0;
    }
    let n = data.len().min(DISK_BYTES - off);
    unsafe {
        DISK[off..off + n].copy_from_slice(&data[..n]);
    }
    n
}

/*
 * file_count - number of DRFS directory entries
 */
pub fn file_count() -> usize {
    if !ready() {
        return 0;
    }
    let mut magic = [0u8; 4];
    unsafe {
        magic.copy_from_slice(&DISK[0..4]);
    }
    if &magic != MAGIC {
        return 0;
    }
    read_u32(8) as usize
}

/*
 * dirent - read directory entry @idx
 */
pub fn dirent(idx: usize) -> Option<DirEnt> {
    let n = file_count();
    if idx >= n || idx >= MAX_FILES {
        return None;
    }
    let base = DIR_OFF + idx * ENTRY_SIZE;
    let mut name = [0u8; NAME_LEN];
    unsafe {
        name.copy_from_slice(&DISK[base..base + NAME_LEN]);
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

/*
 * lookup - find a DRFS file by name; copy up to @out.len() bytes into @out
 *
 * Returns Some((copied_len, total_len, is_text)) or None.
 */
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
