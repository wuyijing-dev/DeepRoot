/*
 * DeepRoot kernel linker script for QEMU virt (RISC-V64, OpenSBI).
 *
 * OpenSBI jumps to the kernel entry at 0x80200000 on the virt machine.
 * Sections stay page-aligned for Sv39 identity mapping (0.2.x).
 */

OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    . = 0x80200000;
    __skernel = .;

    .text : ALIGN(4096) {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : ALIGN(4096) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    .data : ALIGN(4096) {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    .bss : ALIGN(4096) {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        __bss_end = .;
    }

    /*
     * Per-hart kernel stacks (1.7 SMP). Must sit *after* BSS so the boot
     * hart can clear BSS while using hart0's stack. Matches smp::MAX_HARTS
     * (4) * HART_STACK_SIZE (64 KiB).
     */
    . = ALIGN(4096);
    __deeproot_hart_stacks = .;
    . += 4 * 64 * 1024;
    __deeproot_hart_stacks_end = .;

    /* Legacy alias: hart 0 stack top (early trap / single-hart paths). */
    __boot_stack_bottom = __deeproot_hart_stacks;
    __boot_stack_top = __deeproot_hart_stacks + 64 * 1024;

    /*
     * First free physical byte after the loaded image + hart stacks.
     * Frame allocator hands out pages from here upward.
     */
    . = ALIGN(4096);
    __ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
