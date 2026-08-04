//! Teaching filesystem facade (1.3–1.9).
//!
//! Layers:
//! - Embed ramfs ELFs/text (build-time) — root names only
//! - In-RAM VFS tree (1.9) — directories + nested files / scratch
//! - Block DRFS (1.6) — flat on-disk text at root

use crate::block;
use crate::println;
use crate::vfs::{self, Kind, ROOT};

static HELLO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-hello"));
static BADAPPLE_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-badapple"));
static MODDEMO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-moddemo"));

struct File {
    name: &'static str,
    data: &'static [u8],
}

static FILES: &[File] = &[
    File {
        name: "readme.txt",
        data: b"DeepRoot 1.10 - try: modload moddemo; modules; (init also loads it)\n",
    },
    File {
        name: "hello.txt",
        data: b"ELF binary lives at /hello; shell: run hello\n",
    },
    File {
        name: "version",
        data: b"1.10.0\n",
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
];

pub fn init() {
    vfs::init();
    println!("vfs: in-RAM tree ready (mkdir / nested files)");
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
 * lookup - embed ELF/text by root basename (SYS_EXEC)
 */
pub fn lookup(path: &str) -> Option<(&'static str, &'static [u8])> {
    let name = basename(path);
    if name.is_empty() {
        return None;
    }
    /* Only allow embed at root path like "hello" or "/hello". */
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

pub fn mkdir(cwd: usize, path: &str) -> bool {
    if path.is_empty() || path == "/" {
        return false;
    }
    /* Refuse clobbering embed names at root. */
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

/*
 * write_path - create/overwrite a VFS file (supports nested paths)
 */
pub fn write_path(cwd: usize, path: &str, data: &[u8]) -> bool {
    if lookup(path).is_some() {
        return false;
    }
    vfs::write_file(cwd, path, data)
}

/// Compat for 1.8 callers: write at cwd-relative path.
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
                /* Allow listing "/" even if only embeds — use ROOT. */
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

    let mut buf = [0u8; FILE_CAP];
    if let Some(n) = vfs::read_file(cwd, path, &mut buf) {
        if let Ok(s) = core::str::from_utf8(&buf[..n]) {
            crate::console::_print(core::format_args!("{}", s));
        }
        return true;
    }

    /* DRFS only for root-level basenames. */
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

const FILE_CAP: usize = 256;

#[allow(dead_code)]
pub fn cat(path: &str) -> bool {
    cat_at(ROOT, path)
}
