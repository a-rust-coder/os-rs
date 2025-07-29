SECTIONS {
    . = 0x100000;
    .text : ALIGN(4K) { *(.text*) }
    .data : ALIGN(4K) { *(.data*) }
    .bss  : ALIGN(4K) { *(.bss*) }
    .reloc : ALIGN(4K) { KEEP(*(.reloc)) }
}
