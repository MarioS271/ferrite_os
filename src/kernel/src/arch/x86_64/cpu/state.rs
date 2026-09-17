// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 specific CPU data such as TSS, GDT, IDT and selectors
//!
//! Authors: MarioS271

use super::tables;
use super::tables::gdt::{gdt_init, gdt_load};
use super::tables::idt::idt_init;
use crate::arch::x86_64::cpu::gs_info::GsInfo;
use crate::lib::addr::VirtAddr;
use crate::lib::sync::unchecked_cell::UncheckedCell;
use core::sync::atomic::{AtomicU16, Ordering};
use x86_64::structures::gdt::{GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;

/// Global CPU state such as IDT and user code/data selectors
#[repr(align(64))]
pub struct GlobalCpuState {
    idt: UncheckedCell<InterruptDescriptorTable>,
    user_code_selector: AtomicU16,
    user_data_selector: AtomicU16,
}

impl GlobalCpuState {
    /// Constructor; returns a [`GlobalCpuState`] with an uninitialized IDT and zeroed selectors
    pub const fn new() -> Self {
        Self {
            idt: UncheckedCell::new(),
            user_code_selector: AtomicU16::new(0),
            user_data_selector: AtomicU16::new(0),
        }
    }

    /// Initialize the IDT
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_idt(&'static self) {
        // Safety (for idt().load()): IDT was placed in its static location one line above
        unsafe {
            self.idt.init(idt_init());
            self.idt().load();
        }
    }

    /// Set the user data and code selectors
    pub fn set_user_selectors(&self, code: u16, data: u16) {
        self.user_code_selector.store(code, Ordering::Release);
        self.user_data_selector.store(data, Ordering::Release);
    }

    /// Getter for `GlobalCpuState::Idt`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn idt(&self) -> &InterruptDescriptorTable {
        unsafe { self.idt.get() }
    }

    /// Getter for `GlobalCpuState::user_code_selector`
    pub fn user_code_selector(&self) -> u16 {
        self.user_code_selector.load(Ordering::Acquire)
    }

    /// Getter for `GlobalCpuState::user_data_selector`
    pub fn user_data_selector(&self) -> u16 {
        self.user_data_selector.load(Ordering::Acquire)
    }
}


/// Per-CPU state such as TSS and GDT
#[repr(align(64))]
pub struct CpuState {
    tss: tables::tss::Tss,
    gdt: UncheckedCell<GlobalDescriptorTable>,
    gs_info: GsInfo,
}

impl CpuState {
    /// Constructor; returns a [`CpuState`] with an uninitialized TSS and GDT
    pub const fn new() -> Self {
        Self {
            tss: tables::tss::Tss::new(),
            gdt: UncheckedCell::new(),
            gs_info: GsInfo::new()
        }
    }

    /// Initialize the TSS
    ///
    /// # Safety
    /// The caller must guarantee that this method is called exactly once before any calls to
    /// [`CpuState::tss`] are made
    pub unsafe fn init_tss(&'static self) {
        unsafe { self.tss.init() }
    }

    /// Initialize the GDT
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_gdt(&'static self) -> (SegmentSelector, SegmentSelector) {
        let (gdt, gdt_setup_info) = gdt_init(self.tss());

        // Safety (for gdt_load): self.gdt was correctly initialized one line
        // before the call
        unsafe {
            self.gdt.init(gdt);
            gdt_load(self.gdt(), &gdt_setup_info);
        }

        let user_code = gdt_setup_info.user_code;
        let user_data = gdt_setup_info.user_data;

        (user_code, user_data)
    }

    /// Initialize the GS Info Struct
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - This method is called exactly once on the CPU which owns the given [`CpuState`]
    /// - This method is called before the first jump to userspace on the CPU which owns the
    ///   given [`CpuState`]
    pub unsafe fn init_gs_info(&'static self) {
        unsafe { self.gs_info.init() };
    }

    /// Set the kernel stack top of the CPU which owns this [`CpuState`]
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That [`CpuState::init_tss`] has already been called
    /// - This method is only called on the CPU which owns this TSS
    /// - That no refs/ptrs from [`CpuState::tss`] are held when this is called
    /// - `kernel_stack_top` is a valid pointer to a valid, mapped kernel stack
    pub unsafe fn set_kernel_stack_top(&self, kernel_stack_top: VirtAddr) {
        unsafe {
            self.tss.set_rsp0(kernel_stack_top);
            self.gs_info.set_kernel_stack_top(kernel_stack_top);
        };
    }

    /// Getter for `CpuState::tss`
    pub fn tss(&self) -> &tables::tss::Tss {
        &self.tss
    }

    /// Getter for `CpuState::gdt`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn gdt(&self) -> &GlobalDescriptorTable {
        unsafe { self.gdt.get() }
    }

    /// Getter for `CpuState::gs_info`
    pub fn gs_info(&self) -> &GsInfo {
        #[cfg(feature = "debug-checks")]
        if self.gs_info.self_ptr() == 0 {
            kernel_panic(
                PanicCode::UninitializedAccess,
                "Attempted to access an uninitialized GsInfo"
            );
        }

        &self.gs_info
    }
}
