//! Teaching block layer — DRFS on virtio-blk (1.6+) or ramdisk fallback.
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
//! 1.11: [`put_file`] / [`append_file`] mutate the image (create / replace /
//! append). Byte I/O goes through [`virtio_blk`] when present.

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

fn payload_base() -> usize {
    DIR_OFF + MAX_FILES * ENTRY_SIZE
}

fn store_lim() -> usize {
    STORE_SIZE.load(Ordering::Relaxed).min(DRFS_BYTES)
}

fn file_count_raw() -> usize {
    if !has_drfs_magic() {
        return 0;
    }
    (read_u32(8) as usize).min(MAX_FILES)
}

fn dirent_raw(idx: usize) -> Option<DirEnt> {
    let n = file_count_raw();
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

/*
 * payload_raw_end - highest offset+length among dirents (unaligned)
 */
fn payload_raw_end() -> usize {
    let mut end = payload_base();
    for i in 0..file_count_raw() {
        if let Some(ent) = dirent_raw(i) {
            let e = ent.offset as usize + ent.length as usize;
            if e > end {
                end = e;
            }
        }
    }
    end
}

fn used_end() -> usize {
    (payload_raw_end() + 3) & !3
}

fn valid_name(name: &str) -> bool {
    if name.is_empty() || name.len() >= NAME_LEN {
        return false;
    }
    if name.contains('/') || name.contains('\0') {
        return false;
    }
    true
}

fn find_slot(name: &str) -> Option<usize> {
    let n = file_count_raw();
    for i in 0..n {
        if let Some(ent) = dirent_raw(i) {
            if ent.name_str() == name {
                return Some(i);
            }
        }
    }
    None
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

    let data_base = payload_base();
    let files: &[(&str, &[u8])] = &[
        (
            "block.txt",
            b"DeepRoot 1.11.0 - DRFS create/append; root shell > is durable.\n",
        ),
        (
            "from-block",
            b"echo hi > note.txt  # survives QEMU restart on deeproot-disk.img\n",
        ),
        ("blk-version", b"1.11.0\n"),
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
        /* 1.11 smoke: prove a prior boot's durable.txt is still here. */
        let mut probe = [0u8; 32];
        if let Some((n, _, _)) = lookup("durable.txt", &mut probe) {
            const MARK: &[u8] = b"DeepRoot 1.11 durable";
            if n >= MARK.len() && &probe[..MARK.len()] == MARK {
                println!("block: durable.txt survived reboot");
            }
        }
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
    if !ready() {
        return 0;
    }
    file_count_raw()
}

pub fn dirent(idx: usize) -> Option<DirEnt> {
    if !ready() {
        return None;
    }
    dirent_raw(idx)
}

pub fn lookup(name: &str, out: &mut [u8]) -> Option<(usize, usize, bool)> {
    if !has_drfs_magic() {
        return None;
    }
    let n = file_count_raw();
    for i in 0..n {
        let Some(ent) = dirent_raw(i) else {
            continue;
        };
        if ent.name_str() == name {
            let total = ent.length as usize;
            let mut nread = 0usize;
            if !out.is_empty() && total > 0 {
                let cap = out.len().min(total);
                nread = backend_read(ent.offset as usize, &mut out[..cap]);
            }
            return Some((nread, total, ent.is_text()));
        }
    }
    None
}

/*
 * put_file - create or replace a root DRFS file (1.11)
 *
 * Replaces in-place when the new payload fits in the old length; otherwise
 * allocates at used_end (old bytes become dead space until reformat).
 */
pub fn put_file(name: &str, data: &[u8], flags: u32) -> bool {
    if !ready() || !has_drfs_magic() || !valid_name(name) {
        return false;
    }
    let lim = store_lim();
    if data.len() > lim {
        return false;
    }

    if let Some(slot) = find_slot(name) {
        let Some(ent) = dirent_raw(slot) else {
            return false;
        };
        let off = if data.len() <= ent.length as usize {
            ent.offset as usize
        } else {
            let at = used_end();
            if at + data.len() > lim {
                return false;
            }
            at
        };
        if off + data.len() > lim {
            return false;
        }
        put_bytes(off, data);
        write_entry(slot, name, off as u32, data.len() as u32, flags);
        return true;
    }

    let n = file_count_raw();
    if n >= MAX_FILES {
        return false;
    }
    let at = used_end();
    if at + data.len() > lim {
        return false;
    }
    put_bytes(at, data);
    write_entry(n, name, at as u32, data.len() as u32, flags);
    write_u32(8, (n + 1) as u32);
    true
}

/*
 * append_file - append bytes to a DRFS file, or create if missing
 */
pub fn append_file(name: &str, data: &[u8], flags: u32) -> bool {
    if !ready() || !has_drfs_magic() || !valid_name(name) || data.is_empty() {
        return false;
    }
    let lim = store_lim();

    let Some(slot) = find_slot(name) else {
        return put_file(name, data, flags);
    };
    let Some(ent) = dirent_raw(slot) else {
        return false;
    };
    let old_off = ent.offset as usize;
    let old_len = ent.length as usize;
    let end = old_off + old_len;
    let raw_end = payload_raw_end();
    let new_flags = flags | ent.flags;

    /* Contiguous last payload: extend in place. */
    if end == raw_end {
        if end + data.len() > lim {
            return false;
        }
        put_bytes(end, data);
        write_entry(
            slot,
            name,
            old_off as u32,
            (old_len + data.len()) as u32,
            new_flags,
        );
        return true;
    }

    /* Not last: copy old+new to a fresh payload (cap 4 KiB stack). */
    const CAP: usize = 4096;
    if old_len + data.len() > CAP {
        return false;
    }
    let mut buf = [0u8; CAP];
    if old_len > 0 {
        let n = backend_read(old_off, &mut buf[..old_len]);
        if n != old_len {
            return false;
        }
    }
    buf[old_len..old_len + data.len()].copy_from_slice(data);
    let total = old_len + data.len();
    let at = used_end();
    if at + total > lim {
        return false;
    }
    put_bytes(at, &buf[..total]);
    write_entry(slot, name, at as u32, total as u32, new_flags);
    true
}
