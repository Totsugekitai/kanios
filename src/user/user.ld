ENTRY(start)

SECTIONS {
    . = 0x1000000;

    .text : {
        KEEP(*(.text.start));
        *(.text .text.*);
    }

    .rodata : ALIGN(8) {
        *(.rodata .rodata.*);
    }

    .data : ALIGN(8) {
        *(.data .data.*);
    }

    .bss : ALIGN(8) {
        *(.bss .bss.* .sbss .sbss.*);

        . = ALIGN(8);
        . += 64 * 1024; /* 64KB */
        PROVIDE(__stack_top = .);

        ASSERT(. < 0x1800000, "too large executable");
    }
}
