//! modnote — second loadable IPC server (1.10.1).
//!
//! Badge 0xD002. Copy to VFS then modload, or `modload modnote`.

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

pub const MODNOTE_BADGE: u64 = 0xD002;

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("modnote: online\n");
    loop {
        let label = sys::ipc_recv(MODNOTE_BADGE);
        if label < 0 {
            let _ = sys::yield_now();
            continue;
        }
        let _ = sys::debug_write("modnote: noted\n");
        let _ = sys::ipc_reply(MODNOTE_BADGE, 0x4E4F, 1); /* 'NO' */
        let _ = sys::yield_now();
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("modnote: PANIC\n");
    sys::exit(1);
}
