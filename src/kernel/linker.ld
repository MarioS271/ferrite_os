ENTRY(kmain)

PHDRS {
    text    PT_LOAD FLAGS((1 << 0) | (1 << 2));
    rodata  PT_LOAD FLAGS((1 << 2));
    data    PT_LOAD FLAGS((1 << 1) | (1 << 2));
}

__kernel_start = 0xffffffff80000000;

SECTIONS {
    . = __kernel_start;

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

__kernel_text_end = ADDR(.rodata);
__kernel_rodata_end = ADDR(.data);
