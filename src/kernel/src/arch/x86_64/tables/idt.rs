//! arch/x86_64/tables/idt.rs
//! Interrupt Descriptor Table Struct
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::kprint;
use spin::Once;
use x86_64::structures::idt::InterruptDescriptorTable;

const DOUBLE_FAULT_IST_STACK_INDEX: u16 = 0;
const DEBUG_IST_STACK_INDEX: u16 = 1;
const NMI_IST_STACK_INDEX: u16 = 2;
const MACHINE_CHECK_IST_STACK_INDEX: u16 = 3;

static INTERRUPT_DESCRIPTOR_TABLE: Once<InterruptDescriptorTable> = Once::new();

pub fn init() {
    INTERRUPT_DESCRIPTOR_TABLE.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        use crate::arch::x86_64::interrupts::{exceptions, irqs};

        // Faults
        idt.divide_error.set_handler_fn(exceptions::_0_divide_error::handler);
        unsafe { idt.debug.set_handler_fn(exceptions::_1_debug::handler).set_stack_index(DEBUG_IST_STACK_INDEX); }
        unsafe { idt.non_maskable_interrupt.set_handler_fn(exceptions::_2_non_maskable_interrupt::handler).set_stack_index(NMI_IST_STACK_INDEX); }
        idt.breakpoint.set_handler_fn(exceptions::_3_breakpoint::handler);
        idt.overflow.set_handler_fn(exceptions::_4_overflow::handler);
        idt.bound_range_exceeded.set_handler_fn(exceptions::invalid_fault_handler::handler::<5>);
        idt.invalid_opcode.set_handler_fn(exceptions::_6_invalid_opcode::handler);
        idt.device_not_available.set_handler_fn(exceptions::_7_device_not_avail::handler);
        unsafe { idt.double_fault.set_handler_fn(exceptions::_8_double_fault::handler).set_stack_index(DOUBLE_FAULT_IST_STACK_INDEX); }
        idt.invalid_tss.set_handler_fn(exceptions::_10_invalid_tss::handler);
        idt.segment_not_present.set_handler_fn(exceptions::_11_segment_not_present::handler);
        idt.stack_segment_fault.set_handler_fn(exceptions::_12_stack_segment_fault::handler);
        idt.general_protection_fault.set_handler_fn(exceptions::_13_general_protection_fault::handler);
        idt.page_fault.set_handler_fn(exceptions::_14_page_fault::handler);
        idt.x87_floating_point.set_handler_fn(exceptions::_16_x87_floating_point::handler);
        idt.alignment_check.set_handler_fn(exceptions::_17_alignment_check::handler);
        unsafe { idt.machine_check.set_handler_fn(exceptions::_18_machine_check::handler).set_stack_index(MACHINE_CHECK_IST_STACK_INDEX); }
        idt.simd_floating_point.set_handler_fn(exceptions::_19_simd_floating_point::handler);
        idt.virtualization.set_handler_fn(exceptions::_20_virtualization::handler);
        idt.vmm_communication_exception.set_handler_fn(exceptions::_29_vmm_communication_exception::handler);
        idt.security_exception.set_handler_fn(exceptions::_30_security_exception::handler);

        // IRQs
        idt[32].set_handler_fn(irqs::irq0_timer::handler);

        idt
    });

    INTERRUPT_DESCRIPTOR_TABLE.get().unwrap().load();

    kprint!("Initialized INTERRUPT_DESCRIPTOR_TABLE\n");
}
