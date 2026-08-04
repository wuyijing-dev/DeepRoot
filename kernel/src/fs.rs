//! Teaching ramfs (1.3) — in-memory files for shell ls/cat.

use crate::println;

struct File {
    name: &'static str,
    data: &'static [u8],
}

static FILES: &[File] = &[
    File {
        name: "readme.txt",
        data: b"DeepRoot ramfs - teaching filesystem (1.3).\n",
    },
    File {
        name: "hello.txt",
        data: b"spawn the hello ELF with the shell `hello` command.\n",
    },
    File {
        name: "version",
        data: b"1.4.0\n",
    },
];

pub fn list() {
    println!("fs: ramfs /");
    for f in FILES {
        println!("  {} ({} bytes)", f.name, f.data.len());
    }
}

pub fn cat(path: &str) -> bool {
    let name = path.trim_start_matches('/');
    for f in FILES {
        if f.name == name {
            if let Ok(s) = core::str::from_utf8(f.data) {
                crate::console::_print(core::format_args!("{}", s));
            }
            return true;
        }
    }
    println!("fs: no such file '{}'", name);
    false
}
