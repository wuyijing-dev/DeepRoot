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

    /* Boot stack: grows down toward __boot_stack_bottom. */
    . = ALIGN(4096);
    __boot_stack_bottom = .;
    . += 64 * 1024;
    __boot_stack_top = .;

    /*
     * First free physical byte after the loaded image + boot stack.
     * Frame allocator hands out pages from here upward.
     */
    . = ALIGN(4096);
    __ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}
