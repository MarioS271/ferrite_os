// SPDX-License-Identifier: GPL-3.0-only
//! x86_64-specific kernel subsystems: GDT, TSS, IDT, PIC, and interrupt handlers.
//!
//! Call [`init`] once from `kmain` after serial and framebuffer are ready but before
//! interrupts are enabled. The initialization order within `init` is fixed:
//! TSS must exist before the GDT descriptor for it is appended, and the IDT must
//! be loaded before `sti` is issued.
//!
//! Authors: MarioS271

pub(crate) mod instructions;
mod interrupts;
mod tables;

/// Initialize all x86_64 hardware structures required before enabling interrupts.
///
/// Runs in this fixed order:
/// 1. **TSS** — allocates IST stacks and builds the `TaskStateSegment`.
/// 2. **GDT** — appends kernel/user code+data descriptors and the TSS descriptor,
///    loads the table, and reloads all segment registers.
/// 3. **IDT** — registers exception and IRQ handlers (some with IST indices that
///    reference the stacks set up in step 1).
/// 4. **PIC** — remaps IRQ vectors away from the exception range and enables IRQ0.
pub(crate) fn init() {
    tables::tss::init();
    tables::gdt::init();
    tables::idt::init();
    interrupts::pic::init();
}
