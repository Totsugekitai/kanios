ENTRY(boot)

SECTIONS {
    . = 0x80200000;

    .text : {
        KEEP(*(.text.boot));
        *(.text .text.*);
    }

    .rodata : ALIGN(8) {
        *(.rodata .rodata.*);
    }

    .data : ALIGN(8) {
        *(.data .data.*);
    }

    .bss : ALIGN(8) {
        PROVIDE(__bss = .);
        *(.bss .bss.* .sbss .sbss.*);
        PROVIDE(__bss_end = .);
    }

    . = ALIGN(8);
    . += 128 * 1024; /* 128KB */
    PROVIDE(__stack_top = .);
}
