// SPDX-License-Identifier: GPL-3.0-only
//! Per-CPU GS info struct used by the kernel for things like finding the kernel stack on syscall entry
//!
//! Authors: MarioS271

use crate::lib::addr::VirtAddr;
use core::mem::offset_of;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::registers::model_specific::{GsBase, KernelGsBase};

/// A per-CPU struct which each CPU's GS kernel reg points to in order to handle syscall entry
#[repr(C, align(64))]
pub struct GsInfo {
    self_ptr: AtomicU64,
    kernel_stack_top: AtomicU64,
    user_rsp: AtomicU64
}

pub const GS_INFO_SELF_PTR: usize = offset_of!(GsInfo, self_ptr);
pub const GS_INFO_KERNEL_STACK_TOP: usize = offset_of!(GsInfo, kernel_stack_top);
pub const GS_INFO_USER_RSP: usize = offset_of!(GsInfo, user_rsp);

impl GsInfo {
    /// Construct a new zeroed instance of [`GsInfo`]
    pub const fn new() -> Self {
        Self {
            self_ptr: AtomicU64::new(0),
            kernel_stack_top: AtomicU64::new(0),
            user_rsp: AtomicU64::new(0)
        }
    }

    /// Initialize the given [`GsInfo`]
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - This method is called exactly once on the CPU which owns the given [`GsInfo`]
    /// - This method is called before the first jump to userspace on the CPU which owns the
    ///   given [`GsInfo`]
    pub unsafe fn init(&'static self) {
        let self_ptr = VirtAddr::from_ptr(self as *const Self);

        self.self_ptr.store(self_ptr.as_u64(), Ordering::Relaxed);
        self.kernel_stack_top.store(0u64, Ordering::Relaxed);

        KernelGsBase::write(self_ptr.as_x86_64());
        GsBase::write(VirtAddr::null().as_x86_64());
    }

    /// Set the kernel stack top of the given [`GsInfo`]
    ///
    /// # Safety
    /// The caller must ensure that `kernel_stack_top` is a valid pointer to a valid, mapped
    /// kernel stack
    pub unsafe fn set_kernel_stack_top(&self, kernel_stack_top: VirtAddr) {
        self.kernel_stack_top.store(kernel_stack_top.as_u64(), Ordering::Relaxed);
    }

    /// Getter for `GsInfo::self_ptr`
    pub fn self_ptr(&self) -> u64 {
        self.self_ptr.load(Ordering::Relaxed)
    }

    /// Getter for `GsInfo::kernel_stack_top`
    pub fn kernel_stack_top(&self) -> u64 {
        self.kernel_stack_top.load(Ordering::Relaxed)
    }

    /// Getter for `GsInfo::user_rsp`
    pub fn user_rsp(&self) -> u64 {
        self.user_rsp.load(Ordering::Relaxed)
    }
}
