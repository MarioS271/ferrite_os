//! Global Descriptor Table (GDT) initialization.
//!
//! The GDT tells the CPU which memory segments exist and their privilege levels.
//! In 64-bit mode most segmentation is inactive, but the GDT is still required for:
//! - setting the current privilege level (CPL) via the code-segment selector in CS,
//! - telling the CPU where the TSS lives so it can find IST stacks on interrupts.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::kprint;
use spin::Once;
use x86_64::registers::segmentation::{Segment, CS, DS, ES, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{GlobalDescriptorTable, SegmentSelector, Descriptor};

/// Global static holding both the GDT and the selectors returned when descriptors
/// were appended to it. Selectors must remain valid for the lifetime of the GDT,
/// so they are stored together in a single `Once`-initialized bundle.
static GLOBAL_DESCRIPTOR_TABLE: Once<GdtData> = Once::new();

/// Bundles the [`GlobalDescriptorTable`] with all selectors produced by appending
/// descriptors to it, so the selectors stay alive as long as the table does.
struct GdtData {
    global_descriptor_table: GlobalDescriptorTable,
    /// Selector for the kernel-mode code segment; loaded into CS.
    code_selector: SegmentSelector,
    /// Selector for the kernel-mode data segment; loaded into SS, DS, and ES.
    data_selector: SegmentSelector,
    /// Selector for the user-mode code segment (ring 3); reserved for future use.
    user_code_selector: SegmentSelector,
    /// Selector for the user-mode data segment (ring 3); reserved for future use.
    user_data_selector: SegmentSelector,
    /// Selector for the TSS descriptor; loaded via `ltr` so the CPU knows the TSS.
    tss_selector: SegmentSelector,
}

/// Build and load the GDT, then reload all segment registers.
///
/// Appends descriptors in this order: kernel code, kernel data, user code, user data,
/// TSS. After calling `load()`, writes each selector into the corresponding segment
/// register. CS must be set via a far-return or `mov`-equivalent; the x86_64 crate
/// handles this internally in `CS::set_reg`. The TSS is activated by `load_tss`.
///
/// # Panics
/// Panics via `unwrap` if the TSS has not been initialized (i.e., `tss::init()` was
/// not called before `gdt::init()`).
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
