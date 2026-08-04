//! Teaching block device (1.4) — RAM disk standing in for virtio-blk.
//!
//! QEMU virtio-blk can replace this backend later without changing the
//! ramfs / shell path API. For the teaching tree we expose a fixed
//! in-memory "disk" image and a sync hook.

use core::sync::atomic::{AtomicBool, Ordering};

use crate::println;

const DISK_BYTES: usize = 4096;
static mut DISK: [u8; DISK_BYTES] = [0; DISK_BYTES];
static READY: AtomicBool = AtomicBool::new(false);

/*
 * init - fill a tiny disk image and mark the block layer ready
 */
pub fn init() {
    let banner = b"DeepRoot teaching block device (1.4 ramdisk / virtio-blk stand-in)\n";
    unsafe {
        DISK[..banner.len()].copy_from_slice(banner);
    }
    READY.store(true, Ordering::Relaxed);
    println!(
        "block: ramdisk ready size={} (virtio-blk stand-in)",
        DISK_BYTES
    );
}

pub fn ready() -> bool {
    READY.load(Ordering::Relaxed)
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
