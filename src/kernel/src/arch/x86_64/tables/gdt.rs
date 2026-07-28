// SPDX-License-Identifier: GPL-3.0-only
//! Global Descriptor Table (GDT): owns the kernel segment descriptors and the TSS
//! selector, and loads them into the CPU.
//!
//! Authors: MarioS271

use crate::kinfo;
use spin::Once;
use x86_64::registers::segmentation::{Segment, CS, DS, ES, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};

/// Owns the kernel `GlobalDescriptorTable`; must not move after [`Gdt::init`]
/// (the CPU holds its address).
pub struct Gdt {
    table: Once<GlobalDescriptorTable>,
}

impl Gdt {
    /// Create a new, uninitialized `Gdt`; call [`Gdt::init`] to populate and load it.
    pub const fn new() -> Self {
        Self { table: Once::new() }
    }

    /// Build and load the GDT, then reload the segment registers and load the TSS.
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

        kinfo!("Initialized GDT");
    }
}
