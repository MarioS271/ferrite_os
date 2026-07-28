// SPDX-License-Identifier: GPL-3.0-only
//! Interrupt Descriptor Table (IDT): maps every interrupt vector to its handler
//! and loads the table into the CPU.
//!
//! Authors: MarioS271

use crate::kinfo;
use spin::Once;
use x86_64::structures::idt::InterruptDescriptorTable;

/// The single kernel IDT; must not move after loading (the CPU holds its address).
pub struct Idt {
    table: Once<InterruptDescriptorTable>
}
impl Idt {
    /// Create a new, unloaded `Idt`; call [`Idt::init`] to populate and load it.
    pub const fn new() -> Self {
        Self {
            table: Once::new()
        }
    }

    /// Build and load the IDT with all exception and IRQ handlers.
    pub fn init(&'static self) {
        self.table.call_once(|| {
            let mut idt = InterruptDescriptorTable::new();

            use crate::arch::x86_64::interrupts::{exceptions, irqs};
            use super::tss::{DOUBLE_FAULT_IST_STACK_INDEX, DEBUG_IST_STACK_INDEX, NMI_IST_STACK_INDEX, MACHINE_CHECK_IST_STACK_INDEX};

            // Faults
            idt.divide_error.set_handler_fn(exceptions::_0_divide_error::handler);
            unsafe { idt.debug.set_handler_fn(exceptions::_1_debug::handler).set_stack_index(DEBUG_IST_STACK_INDEX as u16); }
            unsafe { idt.non_maskable_interrupt.set_handler_fn(exceptions::_2_non_maskable_interrupt::handler).set_stack_index(NMI_IST_STACK_INDEX as u16); }
            idt.breakpoint.set_handler_fn(exceptions::_3_breakpoint::handler);
            idt.overflow.set_handler_fn(exceptions::_4_overflow::handler);
            idt.bound_range_exceeded.set_handler_fn(exceptions::invalid_fault_handler::handler::<5>);
            idt.invalid_opcode.set_handler_fn(exceptions::_6_invalid_opcode::handler);
            idt.device_not_available.set_handler_fn(exceptions::_7_device_not_avail::handler);
            unsafe { idt.double_fault.set_handler_fn(exceptions::_8_double_fault::handler).set_stack_index(DOUBLE_FAULT_IST_STACK_INDEX as u16); }
            idt.invalid_tss.set_handler_fn(exceptions::_10_invalid_tss::handler);
            idt.segment_not_present.set_handler_fn(exceptions::_11_segment_not_present::handler);
            idt.stack_segment_fault.set_handler_fn(exceptions::_12_stack_segment_fault::handler);
            idt.general_protection_fault.set_handler_fn(exceptions::_13_general_protection_fault::handler);
            idt.page_fault.set_handler_fn(exceptions::_14_page_fault::handler);
            idt.x87_floating_point.set_handler_fn(exceptions::_16_x87_floating_point::handler);
            idt.alignment_check.set_handler_fn(exceptions::_17_alignment_check::handler);
            unsafe { idt.machine_check.set_handler_fn(exceptions::_18_machine_check::handler).set_stack_index(MACHINE_CHECK_IST_STACK_INDEX as u16); }
            idt.simd_floating_point.set_handler_fn(exceptions::_19_simd_floating_point::handler);
            idt.virtualization.set_handler_fn(exceptions::_20_virtualization::handler);
            idt.vmm_communication_exception.set_handler_fn(exceptions::_29_vmm_communication_exception::handler);
            idt.security_exception.set_handler_fn(exceptions::_30_security_exception::handler);

            // IRQs
            idt[32].set_handler_fn(irqs::irq0_timer::handler);

            idt
        });

        self.table.get().unwrap().load();

        kinfo!("Initialized IDT");
    }
}
