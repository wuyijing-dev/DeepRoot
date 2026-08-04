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
        buf[n] = b;
        n += 1;
        let s = core::str::from_utf8(&buf[n - 1..n]).unwrap_or("?");
        let _ = sys::debug_write(s);
    }
    n
}

fn cmd_eq(line: &[u8], cmd: &[u8]) -> bool {
    line == cmd
}

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("shell: DeepRoot shell ready (help, hello, ls, cat, exit)\n");
    let mut buf = [0u8; 64];
    loop {
        let _ = sys::debug_write("deeproot> ");
        let n = read_line(&mut buf);
        let line = &buf[..n];
        if n == 0 {
            continue;
        }
        if cmd_eq(line, b"help") {
            let _ = sys::debug_write("  help   — this text\n");
            let _ = sys::debug_write("  hello  — SYS_SPAWN embedded hello ELF\n");
            let _ = sys::debug_write("  ls     — list ramfs\n");
            let _ = sys::debug_write("  cat X  — read ramfs file X\n");
            let _ = sys::debug_write("  exit   — leave shell\n");
        } else if cmd_eq(line, b"hello") {
            let id = sys::spawn(0);
            let _ = sys::debug_write("shell: spawn hello => ");
            /* print id roughly */
            if id >= 0 {
                let _ = sys::debug_write("ok\n");
            } else {
                let _ = sys::debug_write("fail\n");
            }
            let _ = sys::yield_now();
            let _ = sys::yield_now();
        } else if cmd_eq(line, b"ls") {
            let _ = sys::fs_list();
        } else if line.len() >= 4 && &line[..4] == b"cat " {
            let path = &line[4..];
            let _ = sys::fs_cat(path);
        } else if cmd_eq(line, b"exit") {
            let _ = sys::debug_write("shell: bye\n");
            sys::exit(0);
        } else {
            let _ = sys::debug_write("shell: unknown (try help)\n");
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("shell: PANIC\n");
    sys::exit(1);
}
