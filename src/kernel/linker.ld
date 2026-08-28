ENTRY(kmain)

PHDRS {
    text    PT_LOAD FLAGS((1 << 0) | (1 << 2));
    rodata  PT_LOAD FLAGS((1 << 2));
    data    PT_LOAD FLAGS((1 << 1) | (1 << 2));
}

SECTIONS {
    . = 0xffffffff80000000;

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .data : {
        *(.data .data.*)
    } :data

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data
    
    . = ALIGN(CONSTANT(MAXPAGESIZE));
    
    __kernel_end = .;
}
