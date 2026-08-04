//! Boot glue: entry symbol and BSS clear before `kernel_main`.
//!
//! OpenSBI passes hart id in a0 and a pointer to the DTB in a1.
//! The cold-boot hart may be any id (QEMU/OpenSBI sometimes picks hart 1).
//! Secondary harts enter at `_secondary_start` via SBI HSM (1.7).

core::arch::global_asm!(
    r#"
    .section .text.entry, "ax"
    .globl _start
_start:
    /* a0 = hartid, a1 = dtb — preserve across BSS clear. */
    mv s0, a0
    mv s1, a1
    mv tp, a0

    /* Per-hart stack: base + (hartid+1)<<16 */
    la t0, __deeproot_hart_stacks
    addi t1, a0, 1
    slli t1, t1, 16
    add sp, t0, t1

    /* Clear BSS: [__bss_start, __bss_end). Only cold-boot hart reaches here. */
    la t0, __bss_start
    la t1, __bss_end
1:
    bgeu t0, t1, 2f
    sd zero, 0(t0)
    addi t0, t0, 8
    j 1b
2:
    mv a0, s0
    mv a1, s1
    call kernel_main
3:
    wfi
    j 3b

    .globl _secondary_start
_secondary_start:
    /* HSM: a0=hartid, a1=opaque. satp still 0. */
    mv tp, a0
    la t0, __deeproot_hart_stacks
    addi t1, a0, 1
    slli t1, t1, 16
    add sp, t0, t1
    call secondary_main
5:
    wfi
    j 5b
"#
);

/*
 * Symbols provided by kernel/linker.ld / smp.rs.
 */
unsafe extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
}
