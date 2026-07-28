// SPDX-License-Identifier: GPL-3.0-only
//! Global Descriptor Table (GDT) initialization.
//!
//! The GDT tells the CPU which memory segments exist and their privilege levels.
//! In 64-bit mode most segmentation is inactive, but the GDT is still required for:
//! - setting the current privilege level (CPL) via the code-segment selector in CS,
//! - telling the CPU where the TSS lives so it can find IST stacks on interrupts.
//!
//! Authors: MarioS271

use crate::kprint;
use spin::Once;
use x86_64::registers::segmentation::{Segment, CS, DS, ES, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};

/// Owns the kernel `GlobalDescriptorTable`.
///
/// The instance must not move after [`Gdt::init`] is called — `lgdt` records the
/// table's address and the CPU reads it on every privilege-level transition.
pub struct Gdt {
    table: Once<GlobalDescriptorTable>,
}

impl Gdt {
    /// Create a new, uninitialized `Gdt`; call [`Gdt::init`] to populate and load it.
    pub const fn new() -> Self {
        Self { table: Once::new() }
    }

    /// Build and load the GDT, then reload all segment registers.
    ///
    /// Selectors produced by appending descriptors are used immediately to reload
    /// CS, SS, DS, ES, and the TSS register. CS cannot be set with a normal `mov`;
    /// the x86_64 crate handles the required far-return internally in `CS::set_reg`.
    ///
    /// # Panics
    /// Panics if [`super::tss::Tss::init`] was not called first.
    pub fn init(&'static self, tss: &'static super::tss::Tss) {
        let mut gdt = GlobalDescriptorTable::new();

        let code = gdt.append(Descriptor::kernel_code_segment());
        let data = gdt.append(Descriptor::kernel_data_segment());
        let _ucode = gdt.append(Descriptor::user_code_segment());
        let _udata = gdt.append(Descriptor::user_data_segment());
        let tss_sel = gdt.append(Descriptor::tss_segment(tss.get()));

        self.table.call_once(|| gdt);
        self.table.get().unwrap().load();

        // Safe: the GDT is stored in a static-lifetime Once and will not move;
        // selectors point to valid descriptors appended above.
        unsafe {
            CS::set_reg(code);
            SS::set_reg(data);
            DS::set_reg(data);
            ES::set_reg(data);
            load_tss(tss_sel);
        }

        kprint!("Initialized GDT\n");
    }
}
