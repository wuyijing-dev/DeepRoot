//! Teaching ramfs (1.3) + block-backed text files (1.4.1).
//!
//! Embedded ELFs still come from `kernel/build.rs` via `include_bytes!`.
//! Text files may also live on the DRFS image in `block` (ramdisk stand-in).
//! Shell `ls` / `cat` see both; `run` / `SYS_EXEC` only load ELF from embed.

use crate::block;
use crate::println;

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
        data: b"1.6.0\n",
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

fn normalize(path: &str) -> &str {
    path.trim_start_matches('/')
}

/*
 * lookup - resolve an embedded ramfs path to (name, bytes)
 *
 * Used by SYS_EXEC. Block-backed text files are not returned here.
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

    if !block::ready() {
        return;
    }
    println!("fs: block / (DRFS on ramdisk)");
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

    /* Fall through to block-backed DRFS text files. */
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
