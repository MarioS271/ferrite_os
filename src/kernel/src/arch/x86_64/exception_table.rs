// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 Page Fault Exception Table
//!
//! Authors: MarioS271

unsafe extern "C" {
    static __ex_table_start: u8;
    static __ex_table_end: u8;
}

#[repr(C)]
struct ExceptionTableEntry {
    instruction_offset: i32,
    fixup_offset: i32
}

pub fn search_exception_table(rip: u64) -> Option<u64> {
    let table_start_addr = &raw const __ex_table_start as usize;
    let table_end_addr = &raw const __ex_table_end as usize;

    let num_entries = (table_end_addr - table_start_addr) / size_of::<ExceptionTableEntry>();

    // Safety: the linker script places only 4-byte aligned, 8-byte long entries between the start
    // and end addrs, and the section is read only
    let entries = unsafe {
        core::slice::from_raw_parts((&raw const __ex_table_start).cast::<ExceptionTableEntry>(), num_entries)
    };

    for entry in entries {
        let instruction_field_addr = &raw const entry.instruction_offset as u64;

        if instruction_field_addr.wrapping_add_signed(entry.instruction_offset as i64) != rip {
            continue;
        }

        let fixup_field_addr = &raw const entry.fixup_offset as u64;
        return Some(fixup_field_addr.wrapping_add_signed(entry.fixup_offset as i64));
    }

    None
}
