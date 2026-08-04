OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    . = 0x14000000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : ALIGN(16) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    .data : ALIGN(16) {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    .bss : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        __bss_end = .;
    }
}
