//! console — userspace console server (1.18 tty-ready marker).
//!
//! UART output goes through SYS_DEBUG_WRITE. Init notifies with IPC; shell
//! still uses SYS_DEBUG_READ for the prompt (exclusive UART peel is 1.18.y).

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

const CONSOLE_BADGE: u64 = 0xC001;

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("console: server online\n");
    let _ = sys::debug_write("console: tty read ready\n");
    loop {
        let label = sys::ipc_recv(CONSOLE_BADGE);
        if label < 0 {
            let _ = sys::yield_now();
            continue;
        }
        let _ = sys::debug_write("console: [userspace] message received\n");
        let _ = sys::ipc_reply(CONSOLE_BADGE, label as u64, 0);
        let _ = sys::yield_now();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("console: PANIC\n");
    sys::exit(1);
}
