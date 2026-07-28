// SPDX-License-Identifier: GPL-3.0-only
//! Interrupt-aware spinlock (`IrqMutex`).
//!
//! Authors: MarioS271

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::registers::rflags::RFlags;
use crate::arch::instructions;

/// A spinlock that saves and restores the CPU's Interrupt Flag (RFLAGS.IF).
///
/// When `lock()` is called, the current IF state is captured and interrupts are
/// disabled. When the returned [`IrqMutexGuard`] is dropped, the lock is released
/// and IF is restored to whatever it was before `lock()` was called. This prevents
/// a deadlock that would occur if an interrupt handler tried to acquire a lock
/// already held by the interrupted code.
///
/// Use this type instead of a plain spinlock anywhere that is accessed from both
/// regular kernel code and interrupt handlers.
pub struct IrqMutex<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

unsafe impl<T: Send> Sync for IrqMutex<T> {}
unsafe impl<T: Send> Send for IrqMutex<T> {}

impl<T> IrqMutex<T> {
    /// Create a new unlocked `IrqMutex` wrapping `val`. Usable in `const` context.
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

    /// Release the lock without restoring the interrupt flag.
    ///
    /// Intended for use in panic paths where the normal guard-based release is
    /// unavailable because the guard is owned by the panicking call frame. Clears
    /// the `locked` flag so that subsequent direct writes (which bypass the lock
    /// entirely) are not blocked if the lock happens to be held at panic time.
    ///
    /// # Safety
    /// Calling this while a live [`IrqMutexGuard`] still exists for this mutex
    /// creates two concurrent accessors to the protected data. Only call from a
    /// panic handler that will halt the CPU immediately after and never accesses
    /// the protected data through the mutex again.
    pub unsafe fn force_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

/// RAII guard for [`IrqMutex`].
///
/// Implements [`Deref`] and [`DerefMut`] so the protected value is accessible
/// directly through the guard. When dropped, releases the spinlock and re-enables
/// interrupts if they were enabled at the time `lock()` was called.
pub struct IrqMutexGuard<'g, T> {
    mutex: &'g IrqMutex<T>,
    /// The RFLAGS value captured before `cli` was issued; IF bit determines whether
    /// to call `sti` on drop.
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
