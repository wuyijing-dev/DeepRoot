//! Teaching ramfs (1.3) + block-backed text + scratch overlay (1.8).
//!
//! Embedded ELFs still come from `kernel/build.rs` via `include_bytes!`.
//! Text files may also live on the DRFS image in `block` (ramdisk stand-in).
//! Shell `ls` / `cat` see both; `run` / `SYS_EXEC` only load ELF from embed.
//! `SYS_FS_WRITE` creates/updates small scratch text files in RAM.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::block;
use crate::println;
use crate::sync::SpinLock;

struct File {
    name: &'static str,
    data: &'static [u8],
}

static HELLO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-hello"));
static BADAPPLE_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-badapple"));

static FILES: &[File] = &[
    File {
        name: "readme.txt",
        data: b"DeepRoot ramfs - text + ELF. Try: run hello / cat block.txt\n",
    },
    File {
        name: "hello.txt",
        data: b"ELF binary lives at /hello; shell: run hello\n",
    },
    File {
        name: "version",
        data: b"1.8.0\n",
    },
    File {
        name: "hello",
        data: HELLO_ELF,
    },
    File {
        name: "badapple",
        data: BADAPPLE_ELF,
    },
];

const MAX_SCRATCH: usize = 8;
const SCRATCH_NAME: usize = 24;
const SCRATCH_DATA: usize = 256;

struct Scratch {
    used: bool,
    name_len: usize,
    name: [u8; SCRATCH_NAME],
    data_len: usize,
    data: [u8; SCRATCH_DATA],
}

impl Scratch {
    const fn empty() -> Self {
        Self {
            used: false,
            name_len: 0,
            name: [0; SCRATCH_NAME],
            data_len: 0,
            data: [0; SCRATCH_DATA],
        }
    }

    fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }
}

struct ScratchTable {
    slots: [Scratch; MAX_SCRATCH],
}

struct ScratchCell(UnsafeCell<ScratchTable>);
unsafe impl Sync for ScratchCell {}

static SCRATCH_LOCK: SpinLock = SpinLock::new();
static SCRATCHES: ScratchCell = ScratchCell(UnsafeCell::new(ScratchTable {
    slots: [const { Scratch::empty() }; MAX_SCRATCH],
}));
static SCRATCH_COUNT: AtomicUsize = AtomicUsize::new(0);

fn scratches() -> &'static mut ScratchTable {
    unsafe { &mut *SCRATCHES.0.get() }
}

fn normalize(path: &str) -> &str {
    path.trim_start_matches('/')
}

/*
 * lookup - resolve an embedded ramfs path to (name, bytes)
 *
 * Used by SYS_EXEC. Block-backed / scratch text files are not returned here.
 */
pub fn lookup(path: &str) -> Option<(&'static str, &'static [u8])> {
    let name = normalize(path);
    for f in FILES {
        if f.name == name {
            return Some((f.name, f.data));
        }
    }
    None
}

fn scratch_find(name: &str) -> Option<usize> {
    let t = scratches();
    t.slots.iter().position(|s| s.used && s.name_str() == name)
}

/*
 * write_scratch - create or overwrite a small text file for shell `>`
 */
pub fn write_scratch(path: &str, data: &[u8]) -> bool {
    let name = normalize(path);
    if name.is_empty() || name.len() >= SCRATCH_NAME {
        return false;
    }
    if name.contains('/') || name.contains("..") {
        return false;
    }
    /* Do not clobber embedded ELF names. */
    if lookup(name).is_some() {
        return false;
    }
    let ncopy = data.len().min(SCRATCH_DATA);
    let _g = SCRATCH_LOCK.lock();
    let t = scratches();
    let idx = if let Some(i) = scratch_find(name) {
        i
    } else {
        match t.slots.iter().position(|s| !s.used) {
            Some(i) => {
                SCRATCH_COUNT.fetch_add(1, Ordering::Relaxed);
                i
            }
            None => return false,
        }
    };
    let s = &mut t.slots[idx];
    s.used = true;
    s.name_len = name.len();
    s.name[..name.len()].copy_from_slice(name.as_bytes());
    s.data_len = ncopy;
    s.data[..ncopy].copy_from_slice(&data[..ncopy]);
    true
}

pub fn list() {
    println!("fs: ramfs /");
    for f in FILES {
        let kind = if f.data.len() >= 4 && &f.data[..4] == b"\x7fELF" {
            "elf"
        } else {
            "text"
        };
        println!("  {} ({} bytes, {})", f.name, f.data.len(), kind);
    }

    {
        let _g = SCRATCH_LOCK.lock();
        let t = scratches();
        for s in t.slots.iter() {
            if s.used {
                println!(
                    "  {} ({} bytes, text, scratch)",
                    s.name_str(),
                    s.data_len
                );
            }
        }
    }

    if !block::ready() {
        return;
    }
    println!("fs: block / (DRFS)");
    let n = block::file_count();
    for i in 0..n {
        if let Some(ent) = block::dirent(i) {
            let kind = if ent.is_text() { "text" } else { "bin" };
            println!(
                "  {} ({} bytes, {}, on-disk)",
                ent.name_str(),
                ent.length,
                kind
            );
        }
    }
}

pub fn cat(path: &str) -> bool {
    let name = normalize(path);

    if let Some((fname, data)) = lookup(name) {
        if data.len() >= 4 && &data[..4] == b"\x7fELF" {
            println!(
                "fs: '{}' is ELF ({} bytes) — use: run {}",
                fname,
                data.len(),
                fname
            );
            return true;
        }
        if let Ok(s) = core::str::from_utf8(data) {
            crate::console::_print(core::format_args!("{}", s));
        } else {
            println!("fs: '{}' binary ({} bytes)", fname, data.len());
        }
        return true;
    }

    {
        let _g = SCRATCH_LOCK.lock();
        if let Some(i) = scratch_find(name) {
            let s = &scratches().slots[i];
            if let Ok(text) = core::str::from_utf8(&s.data[..s.data_len]) {
                crate::console::_print(core::format_args!("{}", text));
            }
            return true;
        }
    }

    let mut buf = [0u8; 512];
    match block::lookup(name, &mut buf) {
        Some((nread, total, is_text)) => {
            if !is_text {
                println!("fs: '{}' binary on block ({} bytes)", name, total);
                return true;
            }
            if let Ok(s) = core::str::from_utf8(&buf[..nread]) {
                crate::console::_print(core::format_args!("{}", s));
                if nread < total {
                    println!("fs: … truncated ({} of {} bytes)", nread, total);
                }
            } else {
                println!("fs: '{}' non-utf8 on block ({} bytes)", name, total);
            }
            true
        }
        None => {
            println!("fs: no such file '{}'", name);
            false
        }
    }
}
