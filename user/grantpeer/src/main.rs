//! grantpeer — maps shared page and verifies magic (1.14).

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

pub const GRANTPEER_BADGE: u64 = 0xD014;
const MAGIC: &[u8] = b"DeepRoot 1.14 grant\n";

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("grantpeer: online\n");
    loop {
        let label = sys::ipc_recv(GRANTPEER_BADGE);
        if label < 0 {
            let _ = sys::yield_now();
            continue;
        }
        /* Producer mapped SHARE_VA into our AS before the call. */
        let ok = unsafe {
            let p = sys::SHARE_VA as *const u8;
            let mut i = 0usize;
            while i < MAGIC.len() {
                if *p.add(i) != MAGIC[i] {
                    break;
                }
                i += 1;
            }
            i == MAGIC.len()
        };
        if ok {
            let _ = sys::debug_write("grantpeer: saw magic\n");
            let _ = sys::ipc_reply(GRANTPEER_BADGE, 0x4752, 1); /* 'GR' */
        } else {
            let _ = sys::debug_write("grantpeer: magic mismatch\n");
            let _ = sys::ipc_reply(GRANTPEER_BADGE, 0x4752, 0);
        }
        /* Init will unmap SHARE_VA — exit so we never touch it again. */
        sys::exit(0);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("grantpeer: PANIC\n");
    sys::exit(1);
}
