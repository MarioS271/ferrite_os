//! arch/x86_64/tables/gdt.rs
//! Global Descriptor Table Struct
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::kprint;
use spin::Once;
use x86_64::registers::segmentation::{Segment, CS, DS, ES, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{GlobalDescriptorTable, SegmentSelector, Descriptor};

static GLOBAL_DESCRIPTOR_TABLE: Once<GdtData> = Once::new();

struct GdtData {
    global_descriptor_table: GlobalDescriptorTable,
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    let mut gdt = GlobalDescriptorTable::new();

    let code = gdt.append(Descriptor::kernel_code_segment());
    let data = gdt.append(Descriptor::kernel_data_segment());
    let ucode = gdt.append(Descriptor::user_code_segment());
    let udata = gdt.append(Descriptor::user_data_segment());
    let tss = gdt.append(Descriptor::tss_segment(&super::tss::TASK_STATE_SEGMENT.get().unwrap()));

    let gdt_data = GLOBAL_DESCRIPTOR_TABLE.call_once(|| GdtData{
        global_descriptor_table: gdt,
        code_selector: code,
        data_selector: data,
        user_code_selector: ucode,
        user_data_selector: udata,
        tss_selector: tss,
    });

    gdt_data.global_descriptor_table.load();

    // Safe because the GDT is a valid and initialized static at this point
    // and the selectors point to the correct descriptors
    unsafe {
        CS::set_reg(gdt_data.code_selector);
        SS::set_reg(gdt_data.data_selector);
        DS::set_reg(gdt_data.data_selector);
        ES::set_reg(gdt_data.data_selector);
        load_tss(gdt_data.tss_selector);
    }

    kprint!("Initialized GLOBAL_DESCRIPTOR_TABLE\n");
}
