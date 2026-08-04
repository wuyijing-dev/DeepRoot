//! Boot glue: entry symbol and BSS clear before `kernel_main`.
//!
//! OpenSBI passes hart id in a0 and a pointer to the DTB in a1.

core::arch::global_asm!(
    r#"
    .section .text.entry, "ax"
    .globl _start
_start:
    /* a0 = hartid, a1 = dtb — preserve across BSS clear. */
    mv s0, a0
    mv s1, a1

    /* Stack grows down; linker provides __boot_stack_top. */
    la sp, __boot_stack_top

    /* Clear BSS: [__bss_start, __bss_end). */
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
"#
);

/*
 * Symbols provided by kernel/linker.ld.
 */
unsafe extern "C" {
    static __bss_start: u8;
    static __bss_end: u8;
    static __boot_stack_top: u8;
}
