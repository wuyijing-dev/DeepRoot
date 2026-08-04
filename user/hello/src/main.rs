//! hello — runtime-spawned ELF (1.1).

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
    let _ = sys::debug_write("hello: spawned ELF says hi\n");
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    sys::exit(1);
}
