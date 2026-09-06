// SPDX-License-Identifier: GPL-3.0-only
//! UncheckedCell datatype; a convenience wrapper around `UnsafeCell<MaybeUninit<T>>`
//!
//! Authors: MarioS271

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

#[cfg(feature = "debug-checks")]
use core::sync::atomic::AtomicBool;

/// A datatype with interior mutability, which can be uninitialized via [`MaybeUninit`]
/// When the cargo feature `debug-checks` is enabled, this type also checks for double initialization
/// and uninitialized access.
pub struct UncheckedCell<T> {
    value: UnsafeCell<MaybeUninit<T>>,
    #[cfg(feature = "debug-checks")] is_init: AtomicBool
}

/// Safety: this type can be shared across CPUs as the callers have the responsibility to
/// ensure proper data safety because of the `unsafe` contracts on [`init`](Self::init) and [`get`](Self::get)
unsafe impl<T: Send + Sync> Sync for UncheckedCell<T> {}

impl<T> UncheckedCell<T> {
    /// Constructor; creates a new [`UncheckedCell`]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
            #[cfg(feature = "debug-checks")] is_init: AtomicBool::new(false)
        }
    }

    /// Initialize the inner value
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init(&self, value: T) {
        #[cfg(feature = "debug-checks")]
        {
            use core::sync::atomic::Ordering;
            use crate::lib::panic::kernel_panic;
            use crate::lib::panic_codes::PanicCode;

            if self.is_init.load(Ordering::Relaxed) {
                kernel_panic(
                    PanicCode::DoubleInitialization,
                    "Attempted to double-initialize a UncheckedCell"
                );
            }
        }

        unsafe { (*self.value.get()).write(value) };

        #[cfg(feature = "debug-checks")]
        self.is_init.store(true, Ordering::Relaxed);
    }

    /// Get a reference to the inner value
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn get(&self) -> &T {
        #[cfg(feature = "debug-checks")]
        {
            use core::sync::atomic::Ordering;
            use crate::lib::panic::kernel_panic;
            use crate::lib::panic_codes::PanicCode;

            if !self.is_init.load(Ordering::Relaxed) {
                kernel_panic(
                    PanicCode::UninitializedAccess,
                    "Attempted to access the contents of an uninitialized UncheckedCell"
                );
            }
        }

        unsafe { (*self.value.get()).assume_init_ref() }
    }
}
