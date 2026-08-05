//! Teaching filesystem facade (1.3–1.10).
//!
//! Layers:
//! - Embed ramfs ELFs/text (build-time) — root names only
//! - In-RAM VFS tree (1.9+) — directories + files (FILE_MAX fits small ELFs)
//! - Block DRFS (1.6) — flat on-disk text at root

use core::cell::UnsafeCell;

use crate::block;
use crate::println;
use crate::sync::SpinLock;
use crate::vfs::{self, Kind, FILE_MAX, ROOT};

static HELLO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-hello"));
static BADAPPLE_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-badapple"));
static MODDEMO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-moddemo"));
static MODNOTE_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-modnote"));
static GRANTPEER_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-grantpeer"));
static VIRTIOBLK_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-virtioblk"));

struct File {
    name: &'static str,
    data: &'static [u8],
}

static FILES: &[File] = &[
    File {
        name: "readme.txt",
        data: b"DeepRoot 1.14.3 - userspace virtioblk on hd1; Frame ALLOC_N/PHYS\n",
    },
    File {
        name: "hello.txt",
        data: b"ELF: /hello /virtioblk - peel disk + Frame DMA\n",
    },
    File {
        name: "version",
        data: b"1.14.3\n",
    },
    File {
        name: "hello",
        data: HELLO_ELF,
    },
    File {
        name: "badapple",
        data: BADAPPLE_ELF,
    },
    File {
        name: "moddemo",
        data: MODDEMO_ELF,
    },
    File {
        name: "modnote",
        data: MODNOTE_ELF,
    },
    File {
        name: "grantpeer",
        data: GRANTPEER_ELF,
    },
    File {
        name: "virtioblk",
        data: VIRTIOBLK_ELF,
    },
];

/// Scratch for loading a VFS ELF into SYS_SPAWN_SERVER (not embed).
struct ElfScratch {
    buf: [u8; FILE_MAX],
    name: [u8; 28],
    name_len: usize,
    len: usize,
}

struct ScratchCell(UnsafeCell<ElfScratch>);
unsafe impl Sync for ScratchCell {}

static ELF_LOCK: SpinLock = SpinLock::new();
static ELF_SCRATCH: ScratchCell = ScratchCell(UnsafeCell::new(ElfScratch {
    buf: [0; FILE_MAX],
    name: [0; 28],
    name_len: 0,
    len: 0,
}));

pub fn init() {
    vfs::init();
    println!("vfs: in-RAM tree ready (mkdir / nested files / ELF-sized vfs)");
}

fn basename<'a>(path: &'a str) -> &'a str {
    let p = path.trim_matches('/');
    match p.rfind('/') {
        Some(i) => &p[i + 1..],
        None => p,
    }
}

fn is_root_level(path: &str) -> bool {
    let p = path.trim_start_matches('/');
    !p.is_empty() && !p.contains('/')
}

/*
 * lookup - embed ELF/text by root basename
 */
pub fn lookup(path: &str) -> Option<(&'static str, &'static [u8])> {
    let name = basename(path);
    if name.is_empty() {
        return None;
    }
    let rest = path.trim_start_matches('/');
    if rest.contains('/') {
        return None;
    }
    for f in FILES {
        if f.name == name {
            return Some((f.name, f.data));
        }
    }
    None
}

/*
 * read_bytes - embed / VFS / DRFS into @out; returns byte count
 */
pub fn read_bytes(cwd: usize, path: &str, out: &mut [u8]) -> Option<usize> {
    if let Some((_, data)) = lookup(path) {
        let n = data.len().min(out.len());
        out[..n].copy_from_slice(&data[..n]);
        return Some(n);
    }
    if let Some(n) = vfs::read_file(cwd, path, out) {
        return Some(n);
    }
    if is_root_level(path) {
        let name = basename(path);
        match block::lookup(name, out) {
            Some((nread, _total, _is_text)) => return Some(nread),
            None => {}
        }
    }
    None
}

/*
 * load_elf - resolve path to ELF bytes for spawn (embed or VFS copy in scratch)
 *
 * VFS copies use a single scratch; overwritten on the next load_elf.
 */
pub fn load_elf(cwd: usize, path: &str) -> Option<(&'static str, &'static [u8])> {
    if let Some((name, data)) = lookup(path) {
        if data.len() >= 4 && &data[..4] == b"\x7fELF" {
            return Some((name, data));
        }
        return None;
    }

    let _g = ELF_LOCK.lock();
    let sc = unsafe { &mut *ELF_SCRATCH.0.get() };
    let n = read_bytes(cwd, path, &mut sc.buf)?;
    if n < 4 || &sc.buf[..4] != b"\x7fELF" {
        return None;
    }
    sc.len = n;
    let bn = basename(path);
    let nl = bn.len().min(sc.name.len());
    sc.name[..nl].copy_from_slice(bn.as_bytes());
    sc.name_len = nl;
    let name = core::str::from_utf8(&sc.name[..sc.name_len]).unwrap_or("module");
    let name_ptr = name as *const str;
    let bytes = core::ptr::slice_from_raw_parts(sc.buf.as_ptr(), sc.len);
    /* SAFETY: scratch only overwritten on the next load_elf. */
    Some(unsafe { (&*name_ptr, &*bytes) })
}

/*
 * copy_to_vfs - copy readable src (embed/VFS/DRFS) onto a VFS destination path
 */
pub fn copy_to_vfs(cwd: usize, src: &str, dst: &str) -> bool {
    if lookup(dst).is_some() {
        return false;
    }
    let mut buf = [0u8; FILE_MAX];
    let Some(n) = read_bytes(cwd, src, &mut buf) else {
        return false;
    };
    if n == 0 {
        return false;
    }
    vfs::write_file(cwd, dst, &buf[..n])
}

pub fn mkdir(cwd: usize, path: &str) -> bool {
    if path.is_empty() || path == "/" {
        return false;
    }
    if is_root_level(path) && lookup(path).is_some() {
        return false;
    }
    vfs::mkdir(cwd, path)
}

pub fn rmdir(cwd: usize, path: &str) -> bool {
    vfs::rmdir(cwd, path)
}

pub fn unlink(cwd: usize, path: &str) -> bool {
    if lookup(path).is_some() {
        return false;
    }
    vfs::unlink(cwd, path)
}

pub fn chdir(cwd: usize, path: &str) -> Option<usize> {
    if path.is_empty() {
        return Some(ROOT);
    }
    vfs::chdir(cwd, path)
}

pub fn getcwd(cwd: usize, out: &mut [u8]) -> usize {
    vfs::getcwd(cwd, out)
}

pub fn file_len(cwd: usize, path: &str) -> Option<usize> {
    if let Some((_, data)) = lookup(path) {
        return Some(data.len());
    }
    let mut scratch = [0u8; FILE_MAX];
    if let Some(n) = vfs::read_file(cwd, path, &mut scratch) {
        return Some(n);
    }
    if is_root_level(path) && block::ready() {
        let mut tmp = [0u8; 1];
        if let Some((_, total, _)) = block::lookup(basename(path), &mut tmp) {
            return Some(total);
        }
    }
    None
}

pub fn write_path(cwd: usize, path: &str, data: &[u8]) -> bool {
    if lookup(path).is_some() {
        return false;
    }
    if is_root_level(path) && block::ready() {
        let name = basename(path);
        /* Drop a VFS shadow so later cat/list prefer the on-disk file. */
        let _ = vfs::unlink(cwd, name);
        return block::put_file(name, data, 1);
    }
    vfs::write_file(cwd, path, data)
}

/*
 * append_path - append to DRFS (root) or replace-extend on VFS
 */
pub fn append_path(cwd: usize, path: &str, data: &[u8]) -> bool {
    if lookup(path).is_some() {
        return false;
    }
    if is_root_level(path) && block::ready() {
        let name = basename(path);
        let _ = vfs::unlink(cwd, name);
        return block::append_file(name, data, 1);
    }
    /* VFS: read-modify-write into FILE_MAX. */
    let mut buf = [0u8; FILE_MAX];
    let old = vfs::read_file(cwd, path, &mut buf).unwrap_or(0);
    if old + data.len() > FILE_MAX {
        return false;
    }
    buf[old..old + data.len()].copy_from_slice(data);
    vfs::write_file(cwd, path, &buf[..old + data.len()])
}

#[allow(dead_code)]
pub fn write_scratch(path: &str, data: &[u8]) -> bool {
    write_path(ROOT, path, data)
}

pub fn list_at(cwd: usize, path: Option<&str>) {
    let dir = match path {
        None | Some("") => cwd,
        Some(p) => match vfs::resolve(cwd, p) {
            Some((idx, Kind::Dir)) => idx,
            Some((_, Kind::File)) => {
                println!("fs: not a directory");
                return;
            }
            None => {
                if p == "/" {
                    ROOT
                } else {
                    println!("fs: no such directory '{}'", p);
                    return;
                }
            }
        },
    };

    let mut label = [0u8; 96];
    let ln = vfs::getcwd(dir, &mut label);
    let label_s = core::str::from_utf8(&label[..ln]).unwrap_or("/?");
    println!("fs: {}", label_s);

    if dir == ROOT {
        for f in FILES {
            let kind = if f.data.len() >= 4 && &f.data[..4] == b"\x7fELF" {
                "elf"
            } else {
                "text"
            };
            println!("  {} ({} bytes, {}, embed)", f.name, f.data.len(), kind);
        }
        if block::ready() {
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
    }

    vfs::list_dir(dir, |name, kind, size| match kind {
        Kind::Dir => println!("  {}/", name),
        Kind::File => println!("  {} ({} bytes, vfs)", name, size),
    });
}

#[allow(dead_code)]
pub fn list() {
    list_at(ROOT, Some("/"));
}

pub fn cat_at(cwd: usize, path: &str) -> bool {
    if let Some((fname, data)) = lookup(path) {
        if data.len() >= 4 && &data[..4] == b"\x7fELF" {
            println!(
                "fs: '{}' is ELF ({} bytes) — use: run {} / modload {}",
                fname,
                data.len(),
                fname,
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

    let mut buf = [0u8; FILE_MAX];
    if let Some(n) = vfs::read_file(cwd, path, &mut buf) {
        if n >= 4 && &buf[..4] == b"\x7fELF" {
            println!("fs: VFS ELF ({} bytes) — use: modload {}", n, path);
            return true;
        }
        if let Ok(s) = core::str::from_utf8(&buf[..n]) {
            crate::console::_print(core::format_args!("{}", s));
        }
        return true;
    }

    if is_root_level(path) {
        let name = basename(path);
        let mut bbuf = [0u8; 512];
        match block::lookup(name, &mut bbuf) {
            Some((nread, total, is_text)) => {
                if !is_text {
                    println!("fs: '{}' binary on block ({} bytes)", name, total);
                    return true;
                }
                if let Ok(s) = core::str::from_utf8(&bbuf[..nread]) {
                    crate::console::_print(core::format_args!("{}", s));
                    if nread < total {
                        println!("fs: … truncated ({} of {} bytes)", nread, total);
                    }
                }
                return true;
            }
            None => {}
        }
    }

    println!("fs: no such file '{}'", path);
    false
}

#[allow(dead_code)]
pub fn cat(path: &str) -> bool {
    cat_at(ROOT, path)
}
