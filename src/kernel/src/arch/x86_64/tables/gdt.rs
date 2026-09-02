// SPDX-License-Identifier: GPL-3.0-only
//! Global Descriptor Table (GDT): owns the kernel segment descriptors and the TSS
//! selector, and loads them into the CPU.
//!
//! Authors: MarioS271

use crate::arch::tables::tss::Tss;
use crate::kinfo;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{Segment, CS, DS, ES, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};

pub struct GdtSetupInfo {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub tss_selector: SegmentSelector
}

/// Build the GDT and return it and a struct containing it and all its segment selectors
pub fn gdt_init(tss: &'static Tss) -> (GlobalDescriptorTable, GdtSetupInfo) {
    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code = gdt.append(Descriptor::kernel_code_segment());
    let kernel_data = gdt.append(Descriptor::kernel_data_segment());
    let user_code = gdt.append(Descriptor::user_code_segment());
    let user_data = gdt.append(Descriptor::user_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(unsafe { tss.tss() }));

    (gdt, GdtSetupInfo {
        kernel_code,
        kernel_data,
        user_code,
        user_data,
        tss_selector
    })
}

/// Load the GDT and its selectors
///
/// # Safety
/// The caller must guarantee that the given segment selectors in `gdt_setup_info` are valid
/// and point to the given `gdt`
pub unsafe fn gdt_load(gdt: &'static GlobalDescriptorTable, gdt_setup_info: &GdtSetupInfo) {
    gdt.load();

    unsafe {
        CS::set_reg(gdt_setup_info.kernel_code);
        SS::set_reg(gdt_setup_info.kernel_data);
        DS::set_reg(gdt_setup_info.kernel_data);
        ES::set_reg(gdt_setup_info.kernel_data);
        load_tss(gdt_setup_info.tss_selector);
    }

    kinfo!("Initialized GDT");
}
