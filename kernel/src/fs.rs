//! Teaching ramfs (1.3) — text files + externally built ELFs.
//!
//! User programs are compiled by `kernel/build.rs` into `OUT_DIR`, then
//! embedded here with `include_bytes!`. Shell `run <name>` loads them via
//! `SYS_EXEC` (path spawn), rather than hard-coded blob ids.

use crate::println;

struct File {
    name: &'static str,
    data: &'static [u8],
}

/*
 * HELLO_ELF - RISC-V ET_EXEC built from user/hello (same image SYS_SPAWN 0 uses)
 */
static HELLO_ELF: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-hello"));

static FILES: &[File] = &[
    File {
        name: "readme.txt",
        data: b"DeepRoot ramfs - text + ELF. Try: run hello\n",
    },
    File {
        name: "hello.txt",
        data: b"ELF binary lives at /hello; shell: run hello\n",
    },
    File {
        name: "version",
        data: b"1.4.0\n",
    },
    File {
        name: "hello",
        data: HELLO_ELF,
    },
];

fn normalize(path: &str) -> &str {
    path.trim_start_matches('/')
}

/*
 * lookup - resolve a ramfs path to (name, bytes)
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
}

pub fn cat(path: &str) -> bool {
    match lookup(path) {
        Some((name, data)) => {
            if data.len() >= 4 && &data[..4] == b"\x7fELF" {
                println!("fs: '{}' is ELF ({} bytes) — use: run {}", name, data.len(), name);
                return true;
            }
            if let Ok(s) = core::str::from_utf8(data) {
                crate::console::_print(core::format_args!("{}", s));
            } else {
                println!("fs: '{}' binary ({} bytes)", name, data.len());
            }
            true
        }
        None => {
            println!("fs: no such file '{}'", normalize(path));
            false
        }
    }
}
