// SPDX-License-Identifier: GPL-3.0-only
//! Interrupt-aware spinlock (`IrqMutex`).
//!
//! Authors: MarioS271

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::registers::rflags::RFlags;
use crate::arch::instructions;

// TODO: refactor to be arch-abstract

/// Spinlock for data shared between normal kernel code and interrupt handlers;
/// disables interrupts while held to avoid self-deadlock.
pub struct IrqMutex<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

/// Safety: [`IrqMutex`] is an IRQ-safe spinlock-mutex, meaning data can only be accessed
/// by locking it, therefore making parallel data access impossible
unsafe impl<T: Send> Sync for IrqMutex<T> {}
/// Safety: as long as `T` is [`Send`] (which is forced by the bound), [`IrqMutex`] can also
/// be send because it does not add any non-send components
unsafe impl<T: Send> Send for IrqMutex<T> {}

impl<T> IrqMutex<T> {
    /// Create a new unlocked `IrqMutex` wrapping `value`.
    pub const fn new(value: T) -> Self {
        IrqMutex {
            data: UnsafeCell::new(value),
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

/// RAII guard for [`IrqMutex`]; releases the lock and restores previous interrupts state on drop
pub struct IrqMutexGuard<'g, T> {
    mutex: &'g IrqMutex<T>,
    rflags: RFlags,
}

impl<'g, T> IrqMutexGuard<'g, T> {
    /// Construct a guard associated with `mutex`, saving `rflags` for later restore.
    pub fn new(mutex: &'g IrqMutex<T>, rflags: RFlags) -> IrqMutexGuard<'g, T> {
        IrqMutexGuard{ mutex, rflags }
    }
}

impl<'g, T> Drop for IrqMutexGuard<'g, T> {
    /// Unlocks the mutex and restores interrupt state
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
        if self.rflags.contains(RFlags::INTERRUPT_FLAG) {
            instructions::enable_interrupts();
        }
    }
}

impl<'g, T> Deref for IrqMutexGuard<'g, T> {
    type Target = T;

    /// Returns a reference to the contained data
    fn deref(&self) -> &Self::Target {
        // Safety: the IrqMutexGuard can only exist when the data is locked, meaning concurrent access
        // is impossible
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'g, T> DerefMut for IrqMutexGuard<'g, T> {
    /// Returns a mutable reference to the contained data
    fn deref_mut(&mut self) -> &mut Self::Target {
        // Safety: the IrqMutexGuard can only exist when the data is locked, meaning concurrent access
        // is impossible; additionally, &mut guarantees only one mutable reference
        unsafe { &mut *self.mutex.data.get() }
    }
}
