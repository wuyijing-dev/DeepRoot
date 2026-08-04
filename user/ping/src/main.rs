//! ping — IPC echo server.

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

const PING_BADGE: u64 = 0xE001;

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("ping: server online\n");
    loop {
        let label = sys::ipc_recv(PING_BADGE);
        if label < 0 {
            let _ = sys::yield_now();
            continue;
        }
        let _ = sys::debug_write("ping: pong\n");
        let _ = sys::ipc_reply(PING_BADGE, 0x504F, 1); /* 'PO' */
        let _ = sys::yield_now();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("ping: PANIC\n");
    sys::exit(1);
}
