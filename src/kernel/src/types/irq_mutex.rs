//! types/aligned_stack.rs
//! Mutex which saves/restores IF state
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::registers::rflags::RFlags;
use crate::arch::instructions;

pub struct IrqMutex<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

unsafe impl<T: Send> Sync for IrqMutex<T> {}
unsafe impl<T: Send> Send for IrqMutex<T> {}

impl<T> IrqMutex<T> {
    pub const fn new(val: T) -> Self {
        IrqMutex {
            data: UnsafeCell::new(val),
            locked: AtomicBool::new(false),
        }
    }

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
}


pub struct IrqMutexGuard<'g, T> {
    mutex: &'g IrqMutex<T>,
    rflags: RFlags,
}

impl<'g, T> IrqMutexGuard<'g, T> {
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
