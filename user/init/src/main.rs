//! init — root userspace server (0.5.1).

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

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("init: root server online\n");
    let rc = sys::ipc_call(0, 0x5049, 1);
    let _ = sys::yield_now();
    let _ = sys::debug_write("init: ping call done\n");
    if rc == 0 {
        let _ = sys::debug_write("init: ping accepted\n");
    }
    let _ = sys::ipc_call(1, 0xC045, 0);
    let _ = sys::yield_now();
    let _ = sys::debug_write("init: console notified\n");
    let _ = sys::yield_now();
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("init: PANIC\n");
    sys::exit(1);
}
