ENTRY(kernel_entry)

PHDRS {
    text    PT_LOAD FLAGS((1 << 0) | (1 << 2));
    rodata  PT_LOAD FLAGS((1 << 2));
    data    PT_LOAD FLAGS((1 << 1) | (1 << 2));
}

SECTIONS {
    . = 0xffffffff80000000;
    __kernel_start = .;

    .text : {
        *(.text .text.*)
    } :text

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .eh_frame_hdr : { *(.eh_frame_hdr) } :rodata
    .eh_frame : { *(.eh_frame) } :rodata

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .data : {
        *(.data .data.*)
        *(.got .got.plt)
        *(.data.rel.ro .data.rel.ro.*)
    } :data

    . = ALIGN(CONSTANT(MAXPAGESIZE));

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data
    
    . = ALIGN(CONSTANT(MAXPAGESIZE));
    
    __kernel_end = .;

    .debug_abbrev 0 : { *(.debug_abbrev) }
    .debug_info 0 : { *(.debug_info) }
    .debug_aranges 0 : { *(.debug_aranges) }
    .debug_ranges 0 : { *(.debug_ranges) }
    .debug_str 0 : { *(.debug_str) }
    .debug_frame 0 : { *(.debug_frame) }
    .debug_line 0 : { *(.debug_line) }
    .debug_loc 0 : { *(.debug_loc) }

    .symtab 0 : { *(.symtab) }
    .shstrtab 0 : { *(.shstrtab) }
    .strtab 0 : { *(.strtab) }

    /DISCARD/ : { *(.comment) }
}

__kernel_text_end = ADDR(.rodata);
__kernel_rodata_end = ADDR(.data);
