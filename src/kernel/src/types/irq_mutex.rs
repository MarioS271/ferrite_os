// SPDX-License-Identifier: GPL-3.0-only
//! Interrupt-aware spinlock (`IrqMutex`).
//!
//! Authors: MarioS271

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::registers::rflags::RFlags;
use crate::arch::instructions;

/// Spinlock for data shared between normal kernel code and interrupt handlers;
/// disables interrupts while held to avoid self-deadlock.
pub struct IrqMutex<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

unsafe impl<T: Send> Sync for IrqMutex<T> {}
unsafe impl<T: Send> Send for IrqMutex<T> {}

impl<T> IrqMutex<T> {
    /// Create a new unlocked `IrqMutex` wrapping `val`.
    pub const fn new(val: T) -> Self {
        IrqMutex {
            data: UnsafeCell::new(val),
            locked: AtomicBool::new(false),
        }
    }

    /// Acquire the lock, disabling interrupts first. Returns a guard that restores IF on drop.
    pub fn lock(&self) -> IrqMutexGuard<'_, T> {
        let rflags = x86_64::registers::rflags::read();
        instructions::disable_interrupts();

        loop {
            let res = self.locked.compare_exchange(
                false,
                true,
                Ordering::Acquire,
                Ordering::Relaxed
            );
            if res.is_ok() { break; }
        }

        IrqMutexGuard::new(self, rflags)
    }

    /// Release the lock without restoring the interrupt flag, for use in panic paths.
    ///
    /// # Safety
    /// Calling this while a live [`IrqMutexGuard`] still exists creates two concurrent
    /// accessors to the protected data. Only call from a panic handler that halts the
    /// CPU immediately after and never touches the data through the mutex again.
    pub unsafe fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// RAII guard for [`IrqMutex`]; releases the lock and restores the interrupt flag on drop.
pub struct IrqMutexGuard<'g, T> {
    mutex: &'g IrqMutex<T>,
    /// RFLAGS captured before interrupts were disabled; its IF bit decides whether to re-enable on drop.
    rflags: RFlags,
}

impl<'g, T> IrqMutexGuard<'g, T> {
    /// Construct a guard associated with `mutex`, saving `rflags` for later restore.
    pub fn new(mutex: &'g IrqMutex<T>, rflags: RFlags) -> IrqMutexGuard<'g, T> {
        IrqMutexGuard{ mutex, rflags }
    }
}

impl<'g, T> Drop for IrqMutexGuard<'g, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
        if self.rflags.contains(RFlags::INTERRUPT_FLAG) {
            instructions::enable_interrupts();
        }
    }
}

impl<'g, T> Deref for IrqMutexGuard<'g, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'g, T> DerefMut for IrqMutexGuard<'g, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}
