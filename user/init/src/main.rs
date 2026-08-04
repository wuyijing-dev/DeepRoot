//! init — root server: IPC demos, load optional module, hand off to shell.

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

/// Must match user/moddemo and SYS_SPAWN_SERVER badge.
const MODDEMO_BADGE: u64 = 0xD001;
/// Must match user/modnote (1.10.1 VFS load demo).
const MODNOTE_BADGE: u64 = 0xD002;

#[no_mangle]
pub extern "C" fn main() {
    let _ = sys::debug_write("init: root server online\n");
    let rc = sys::ipc_call(0, 0x5049, 1);
    let _ = sys::debug_write("init: ping call done\n");
    if rc >= 0 {
        let _ = sys::debug_write("init: ping accepted\n");
    }
    let _ = sys::ipc_call(1, 0xC045, 0);
    let _ = sys::debug_write("init: console notified\n");
    let hid = sys::spawn(0);
    if hid >= 0 {
        let _ = sys::debug_write("init: spawned hello ELF\n");
    }

    /* 1.10: load optional server from embed path (not part of bring_up canopy). */
    let slot = sys::spawn_server(b"moddemo", MODDEMO_BADGE);
    if slot >= 0 {
        let _ = sys::debug_write("init: module loaded\n");
        let _ = sys::yield_now();
        let _ = sys::yield_now();
        let mrc = sys::ipc_call(slot as usize, 0x4D44, 1); /* 'MD' */
        if mrc >= 0 {
            let _ = sys::debug_write("init: module call ok\n");
        } else {
            let _ = sys::debug_write("init: module call failed\n");
        }
    } else {
        let _ = sys::debug_write("init: module load failed\n");
    }

    /*
     * 1.10.1: copy ELF into VFS, then SYS_SPAWN_SERVER from that file
     * (not only embed basename).
     */
    if sys::fs_cp(b"modnote", b"mynote") >= 0 {
        let _ = sys::debug_write("init: cp modnote -> mynote ok\n");
        let nslot = sys::spawn_server(b"mynote", MODNOTE_BADGE);
        if nslot >= 0 {
            let _ = sys::debug_write("init: vfs module loaded\n");
            let _ = sys::yield_now();
            let _ = sys::yield_now();
            let nrc = sys::ipc_call(nslot as usize, 0x4E4F, 1); /* 'NO' label */
            if nrc >= 0 {
                let _ = sys::debug_write("init: vfs module call ok\n");
            } else {
                let _ = sys::debug_write("init: vfs module call failed\n");
            }
        } else {
            let _ = sys::debug_write("init: vfs module load failed\n");
        }
    } else {
        let _ = sys::debug_write("init: cp modnote failed\n");
    }

    let _ = sys::yield_now();
    let _ = sys::yield_now();
    let _ = sys::debug_write("init: handing off to shell\n");
    sys::exit(0);
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    let _ = sys::debug_write("init: PANIC\n");
    sys::exit(1);
}
