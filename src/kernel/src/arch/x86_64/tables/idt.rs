//! arch/x86_64/tables/idt.rs
//! Interrupt Descriptor Table Struct
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use crate::kprint;
use spin::Once;
use x86_64::structures::idt::InterruptDescriptorTable;

static INTERRUPT_DESCRIPTOR_TABLE: Once<InterruptDescriptorTable> = Once::new();

pub fn init() {
    INTERRUPT_DESCRIPTOR_TABLE.call_once(|| {
        let mut idt = InterruptDescriptorTable::new();

        
        unsafe {
            idt.double_fault.set_handler_fn(super::exceptions::double_fault::handler).set_stack_index(0);
        }
        idt.general_protection_fault.set_handler_fn(super::exceptions::general_protection_fault::handler);
        idt.page_fault.set_handler_fn(super::exceptions::page_fault::handler);
        use crate::arch::x86_64::interrupts;

        // Faults
        unsafe { idt.double_fault.set_handler_fn(interrupts::faults::double_fault::handler).set_stack_index(0); }
        idt.general_protection_fault.set_handler_fn(interrupts::faults::general_protection_fault::handler);
        idt.page_fault.set_handler_fn(interrupts::faults::page_fault::handler);

        idt
    });

    INTERRUPT_DESCRIPTOR_TABLE.get().unwrap().load();

    kprint!("Initialized INTERRUPT_DESCRIPTOR_TABLE\n");
}
