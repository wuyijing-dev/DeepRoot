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

    /* 1.13: resolve canopy by name (not only hard-coded slot 0). */
    let ping_slot = sys::service_lookup(b"ping");
    if ping_slot >= 0 {
        let rc = sys::ipc_call(ping_slot as usize, 0x5049, 1);
        let _ = sys::debug_write("init: lookup ping ok\n");
        let _ = sys::debug_write("init: ping call done\n");
        if rc >= 0 {
            let _ = sys::debug_write("init: ping accepted\n");
        }
    } else {
        let rc = sys::ipc_call(0, 0x5049, 1);
        let _ = sys::debug_write("init: ping call done\n");
        if rc >= 0 {
            let _ = sys::debug_write("init: ping accepted\n");
        }
    }

    let cons = sys::service_lookup(b"console");
    if cons >= 0 {
        let _ = sys::ipc_call(cons as usize, 0xC045, 0);
    } else {
        let _ = sys::ipc_call(1, 0xC045, 0);
    }
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

    /* 1.13: look up loaded module by name and call again. */
    let look = sys::service_lookup(b"mynote");
    if look >= 0 {
        let _ = sys::yield_now();
        let lrc = sys::ipc_call(look as usize, 0x4E4F, 1);
        if lrc >= 0 {
            let _ = sys::debug_write("init: lookup mynote ok\n");
        } else {
            let _ = sys::debug_write("init: lookup mynote call failed\n");
        }
    } else {
        let _ = sys::debug_write("init: lookup mynote failed\n");
    }

    /* 1.11: seed a durable DRFS file (survives QEMU restart on disk.img). */
    const DURABLE: &[u8] = b"DeepRoot 1.11 durable\n";
    if sys::fs_write(b"durable.txt", DURABLE) >= 0 {
        let _ = sys::debug_write("init: durable DRFS written\n");
    } else {
        let _ = sys::debug_write("init: durable DRFS write failed\n");
    }

    /* 1.11.1: open/read/close durable.txt via fds */
    let fd = sys::open(b"durable.txt", sys::O_RDONLY);
    if fd >= 0 {
        let mut buf = [0u8; 32];
        let n = sys::fd_read(fd as usize, &mut buf);
        let _ = sys::close(fd as usize);
        if n >= 19 {
            let _ = sys::debug_write("init: fd read ok\n");
        } else {
            let _ = sys::debug_write("init: fd read short\n");
        }
    } else {
        let _ = sys::debug_write("init: fd open failed\n");
    }

    /* 1.12: sleep + ledger/cap inspect markers */
    let _ = sys::sleep_ms(5);
    let _ = sys::debug_write("init: slept\n");
    let t = sys::time_ms();
    let _ = sys::debug_write("init: time_ms ok\n");
    let _ = t;
    let _ = sys::ledger_dump();
    let _ = sys::debug_write("init: ledger dumped\n");
    let _ = sys::cap_dump();
    let _ = sys::debug_write("init: caps dumped\n");

    /* Drivers first (userspace servers under drivers/) — finish before grant demo. */
    const VIRTIOBLK_BADGE: u64 = 0xD015;
    if sys::spawn_server(b"virtioblk", VIRTIOBLK_BADGE) >= 0 {
        let mut i = 0usize;
        while i < 48 {
            let _ = sys::yield_now();
            i += 1;
        }
        let _ = sys::debug_write("init: virtioblk loaded\n");
    } else {
        let _ = sys::debug_write("init: virtioblk load failed\n");
    }

    const FBDEMO_BADGE: u64 = 0xD016;
    if sys::spawn_server(b"fbdemo", FBDEMO_BADGE) >= 0 {
        let mut i = 0usize;
        while i < 48 {
            let _ = sys::yield_now();
            i += 1;
        }
        let _ = sys::debug_write("init: fbdemo loaded\n");
    } else {
        let _ = sys::debug_write("init: fbdemo load failed\n");
    }

    const FBMENU_BADGE: u64 = 0xD017;
    if sys::spawn_server(b"fbmenu", FBMENU_BADGE) >= 0 {
        let mut i = 0usize;
        while i < 64 {
            let _ = sys::yield_now();
            i += 1;
        }
        let _ = sys::debug_write("init: fbmenu loaded\n");
    } else {
        let _ = sys::debug_write("init: fbmenu load failed\n");
    }

    /* 1.14: shared frame — map into grantpeer, peer verifies magic. */
    const GRANTPEER_BADGE: u64 = 0xD014;
    let gpslot = sys::spawn_server(b"grantpeer", GRANTPEER_BADGE);
    if gpslot >= 0 {
        let _ = sys::yield_now();
        let _ = sys::yield_now();
        let sid = sys::service_sched(b"grantpeer");
        let fslot = sys::frame_alloc();
        if sid >= 0 && fslot >= 0 {
            if sys::frame_map(fslot as usize, sys::SHARE_VA, true) >= 0 {
                unsafe {
                    let p = sys::SHARE_VA as *mut u8;
                    let magic = b"DeepRoot 1.14 grant\n";
                    for (i, b) in magic.iter().enumerate() {
                        *p.add(i) = *b;
                    }
                }
                let _ = sys::frame_map_into(fslot as usize, sid as usize, sys::SHARE_VA, false);
                let _ = sys::frame_grant(fslot as usize, sid as usize, false);
                let _ = sys::yield_now();
                let grc = sys::ipc_call(gpslot as usize, 0x4752, 1);
                if grc >= 0 {
                    let _ = sys::debug_write("init: grant peer ok\n");
                } else {
                    let _ = sys::debug_write("init: grant peer call failed\n");
                }
                /* 1.14.1: unmap peer + self, then revoke Frame (frees PA). */
                let _ = sys::frame_unmap_into(sid as usize, sys::SHARE_VA);
                let _ = sys::frame_unmap(sys::SHARE_VA);
                if sys::cap_revoke(fslot as usize) >= 0 {
                    let _ = sys::debug_write("init: frame revoke ok\n");
                } else {
                    let _ = sys::debug_write("init: frame revoke failed\n");
                }
            } else {
                let _ = sys::debug_write("init: frame map failed\n");
            }
        } else {
            let _ = sys::debug_write("init: grant setup failed\n");
        }
    } else {
        let _ = sys::debug_write("init: grantpeer load failed\n");
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
