//! arch/gdt.rs
//! Global Descriptor Table Struct
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::kprint;
use spin::Once;
use x86_64::registers::segmentation::{CS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{GlobalDescriptorTable, SegmentSelector, Descriptor};

static GLOBAL_DESCRIPTOR_TABLE: Once<GdtData> = Once::new();

struct GdtData {
    global_descriptor_table: GlobalDescriptorTable,
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    let mut gdt = GlobalDescriptorTable::new();

    let code = gdt.append(Descriptor::kernel_code_segment());
    gdt.append(Descriptor::kernel_data_segment());
    let tss = gdt.append(Descriptor::tss_segment(&super::tss::TASK_STATE_SEGMENT.get().unwrap()));

    let gdt_data = GLOBAL_DESCRIPTOR_TABLE.call_once(|| GdtData{
        global_descriptor_table: gdt,
        code_selector: code,
        tss_selector: tss,
    });

    gdt_data.global_descriptor_table.load();

    // Safe because the GDT is a valid and initialized static at this point
    // and the selectors point to the correct descriptors
    unsafe {
        CS::set_reg(gdt_data.code_selector);
        load_tss(gdt_data.tss_selector);
    }

    kprint!("Initialized GLOBAL_DESCRIPTOR_TABLE\n");
}