//! Supervisor Binary Interface (SBI) wrappers — OpenSBI on QEMU virt.
//!
//! Spec reference: RISC-V SBI specification (Legacy + Base + Debug Console).
//! We prefer the modern Debug Console extension when available, with a
//! legacy `console_putchar` fallback for older firmware.

#![allow(dead_code)]

const SBI_EXT_BASE: usize = 0x10;
const SBI_EXT_DBCN: usize = 0x4442434E; /* "DBCN" */
/* Legacy 0.1 console: EID is the operation (fid/a6 ignored). */
const SBI_EXT_0_1_CONSOLE_PUTCHAR: usize = 0x01;
const SBI_EXT_0_1_CONSOLE_GETCHAR: usize = 0x02;
const SBI_EXT_HSM: usize = 0x48534D; /* "HSM" */

const SBI_DBCN_CONSOLE_WRITE_BYTE: usize = 2;
const SBI_HSM_HART_STOP: usize = 1;

#[repr(C)]
struct Sbiret {
    error: isize,
    value: isize,
}

/*
 * sbi_call - invoke an SBI ecall with up to three arguments
 * @ext: extension ID
 * @fid: function ID within the extension
 * @arg0..arg2: passed in a0..a2 per SBI calling convention
 *
 * Returns the raw Sbiret { error, value }. Negative error means failure.
 */
#[inline(always)]
fn sbi_call(ext: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> Sbiret {
    let mut error: isize;
    let mut value: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") ext,
            in("a6") fid,
            inout("a0") arg0 => error,
            inout("a1") arg1 => value,
            in("a2") arg2,
            options(nostack),
        );
    }
    Sbiret { error, value }
}

/*
 * console_putchar - write one byte to the early console
 *
 * Tries SBI Debug Console write_byte first; falls back to legacy putchar.
 * Early boot and panic paths depend on this remaining allocation-free.
 */
pub fn console_putchar(c: u8) {
    let r = sbi_call(SBI_EXT_DBCN, SBI_DBCN_CONSOLE_WRITE_BYTE, c as usize, 0, 0);
    if r.error != 0 {
        /* Legacy putchar: a7=0x01, character in a0; a6 ignored. */
        let _ = sbi_call(SBI_EXT_0_1_CONSOLE_PUTCHAR, 0, c as usize, 0, 0);
    }
}

/*
 * console_getchar - poll one byte from the console (legacy SBI 0.1)
 *
 * EID must be 0x02 (not putchar 0x01). OpenSBI returns the character
 * in a0, or a negative value when no input is pending — not the modern
 * {error,value} pair (and a1 is left untouched for 0.1 calls).
 */
pub fn console_getchar() -> Option<u8> {
    let r = sbi_call(SBI_EXT_0_1_CONSOLE_GETCHAR, 0, 0, 0, 0);
    /* a0 lands in r.error for the legacy ABI. */
    if r.error >= 0 && r.error <= 255 {
        Some(r.error as u8)
    } else {
        None
    }
}

/*
 * hart_suspend_idle - low-power wait; used by the idle loop and panic
 *
 * Prefer WFI locally. HSM stop is intentional for "halt this hart" later.
 */
pub fn hart_suspend_idle() {
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack));
    }
    let _ = (SBI_EXT_BASE, SBI_EXT_HSM, SBI_HSM_HART_STOP);
}
