//! shell — interactive DeepRoot-native shell (1.2+).

#![no_std]
#![no_main]

use deeproot_user::sys;

core::arch::global_asm!(
    r#"
    .section .text.entry, "ax"
    .globl _start
_start:
    la t0, __bss_start
    la t1, __bss_end
1:
    bgeu t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:
    call main
    li a0, 0
    li a7, 9
    ecall
3:
    wfi
    j 3b
"#
);

fn read_line(buf: &mut [u8]) -> usize {
    let mut n = 0usize;
    while n < buf.len() {
        let c = sys::debug_read_byte();
        if c < 0 {
            let _ = sys::yield_now();
            continue;
        }
        let b = c as u8;
        if b == b'\r' || b == b'\n' {
            let _ = sys::debug_write("\n");
            break;
        }
        if b == 0x7f || b == 8 {
            if n > 0 {
                n -= 1;
                let _ = sys::debug_write("\x08 \x08");
            }
            continue;
        }
        /* Ignore other controls. */
        if b < 0x20 {
            continue;
        }
        buf[n] = b;
        n += 1;
        let s = core::str::from_utf8(&buf[n - 1..n]).unwrap_or("?");
        let _ = sys::debug_write(s);
    }
    n
}

fn trim(mut line: &[u8]) -> &[u8] {
    while matches!(line.first().copied(), Some(b' ' | b'\t')) {
        line = &line[1..];
    }
    while matches!(line.last().copied(), Some(b' ' | b'\t')) {
        line = &line[..line.len() - 1];
    }
    line
}

fn cmd_eq(line: &[u8], cmd: &[u8]) -> bool {
    trim(line) == cmd
}

fn run_path(path: &[u8]) {
    let id = sys::exec(path);
    if id < 0 {
        let _ = sys::debug_write("shell: exec failed\n");
        return;
    }
    /* Wait until the child exits so long players (badapple) own the console. */
    loop {
        let st = sys::wait(id as usize);
        if st != -11 {
            break;
        }
        let _ = sys::yield_now();
    }
}

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("shell: DeepRoot shell ready (help, ls, cat, run, exit)\n");
    let mut buf = [0u8; 64];
    loop {
        let _ = sys::debug_write("deeproot> ");
        let n = read_line(&mut buf);
        let line = trim(&buf[..n]);
        if line.is_empty() {
            continue;
        }
        if cmd_eq(line, b"help") {
            let _ = sys::debug_write("  help      - this text\n");
            let _ = sys::debug_write("  ls        - list ramfs\n");
            let _ = sys::debug_write("  cat X     - read text file X\n");
            let _ = sys::debug_write("  run X     - SYS_EXEC ELF from ramfs (hello, badapple)\n");
            let _ = sys::debug_write("  hello     - same as: run hello\n");
            let _ = sys::debug_write("  badapple  - realtime ASCII Bad Apple (q quits)\n");
            let _ = sys::debug_write("  exit      - leave shell\n");
        } else if cmd_eq(line, b"ls") {
            let _ = sys::fs_list();
        } else if line.len() >= 4 && line.starts_with(b"cat ") {
            let path = trim(&line[4..]);
            let _ = sys::fs_cat(path);
        } else if line.len() >= 4 && line.starts_with(b"run ") {
            let path = trim(&line[4..]);
            if path.is_empty() {
                let _ = sys::debug_write("shell: run <elf>\n");
            } else {
                run_path(path);
            }
        } else if cmd_eq(line, b"hello") {
            run_path(b"hello");
        } else if cmd_eq(line, b"badapple") {
            run_path(b"badapple");
        } else if cmd_eq(line, b"exit") {
            let _ = sys::debug_write("shell: bye\n");
            sys::exit(0);
        } else {
            let _ = sys::debug_write("shell: unknown - type: help\n");
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("shell: PANIC\n");
    sys::exit(1);
}
