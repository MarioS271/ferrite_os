// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 specific CPU data such as TSS, GDT, IDT and selectors
//!
//! Authors: MarioS271

use crate::arch::tables;
use crate::arch::tables::gdt::{gdt_init, gdt_load};
use crate::arch::tables::idt::idt_init;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU16, Ordering};
use x86_64::structures::gdt::{GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::idt::InterruptDescriptorTable;

/// Global CPU state such as IDT and user code/data selectors
pub struct GlobalCpuState {
    idt: UnsafeCell<MaybeUninit<InterruptDescriptorTable>>,
    user_code_selector: AtomicU16,
    user_data_selector: AtomicU16,
}

/// Safety:
/// - `idt` is only written to once in `init_idt`; afterwards, no mutable references/pointers will be held to it
/// - `user_code_selector`/`user_data_selector` are [`AtomicU16`] which are [`Sync`]
unsafe impl Sync for GlobalCpuState {}

impl GlobalCpuState {
    /// Constructor; returns a [`GlobalCpuState`] with an uninitialized IDT and zeroed selectors
    pub const fn new() -> Self {
        Self {
            idt: UnsafeCell::new(MaybeUninit::uninit()),
            user_code_selector: AtomicU16::new(0),
            user_data_selector: AtomicU16::new(0),
        }
    }

    /// Initialize the IDT
    ///
    /// # Safety
    /// The caller must ensure that this method is called exactly once before interrupts, SMP
    /// and threading are enabled
    pub unsafe fn init_idt(&'static self) {
        // Safety: IDT was placed in its static location one line above
        unsafe {
            (*self.idt.get()).write(idt_init());
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
    /// This method derefs `self.idt.get()` and calls [`MaybeUninit::assume_init_ref()`].
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - `self.idt` must have been initialized first (via `init_idt`)
    /// - There must be no mutable references or pointers to `self.idt`
    pub unsafe fn idt(&self) -> &InterruptDescriptorTable {
        unsafe { (*self.idt.get()).assume_init_ref() }
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
pub struct CpuState {
    tss: tables::tss::Tss,
    gdt: UnsafeCell<MaybeUninit<GlobalDescriptorTable>>,
}

/// Safety:
/// - `gdt` is only written to once in `init_gdt`; afterwards, no mutable references/pointers will be held to it
/// - `tss` is a [`Tss`](tables::tss::Tss) which is [`Sync`]
unsafe impl Sync for CpuState {}

impl CpuState {
    /// Constructor; returns a [`CpuState`] with an uninitialized TSS and GDT
    pub const fn new() -> Self {
        Self {
            tss: tables::tss::Tss::new(),
            gdt: UnsafeCell::new(MaybeUninit::uninit())
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
    /// The caller must guarantee that:
    /// - [`CpuState::init_tss`] was called before this method
    /// - This method is called exactly once before any calls to [`CpuState::gdt`] are made
    /// - No SMP/threading is active when this method is called
    pub unsafe fn init_gdt(&'static self) -> (SegmentSelector, SegmentSelector) {
        let (gdt, gdt_setup_info) = gdt_init(self.tss());

        // Safety: gdt_load with self.gdt(): self.gdt was correctly initialized one line
        // before the call
        unsafe {
            (*self.gdt.get()).write(gdt);
            gdt_load(self.gdt(), &gdt_setup_info);
        }

        let user_code = gdt_setup_info.user_code;
        let user_data = gdt_setup_info.user_data;

        (user_code, user_data)
    }

    /// Getter for `CpuState::tss`
    pub fn tss(&self) -> &tables::tss::Tss {
        &self.tss
    }

    /// Getter for `CpuState::gdt`
    ///
    /// # Safety
    /// This method derefs `self.gdt.get()` and calls [`MaybeUninit::assume_init_ref()`].
    /// The following conditions **must** be met to avoid undefined behavior when calling this method:
    /// - `self.gdt` must have been initialized first (via `init_gdt`)
    /// - There must be no mutable references or pointers to `self.gdt`
    pub unsafe fn gdt(&self) -> &GlobalDescriptorTable {
        unsafe { (*self.gdt.get()).assume_init_ref() }
    }
}
