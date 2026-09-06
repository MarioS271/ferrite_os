// SPDX-License-Identifier: GPL-3.0-only
//! NicheCell datatype; a wrapper around `UnsafeCell<Option<T>>` which
//! is designed to take advantage of niche optimization
//!
//! Authors: MarioS271

use core::cell::UnsafeCell;

/// A datatype with interior mutability, which can be uninitialized via [`Option`].
/// This datatype is designed to be used with values that can take advantage of niche
/// optimization (where the datatype `T` has a value which is guaranteed to be invalid, like
/// a [`NonZeroU64`](core::num::NonZeroU64) being zero)
///
/// The caller may also use this datatype if [`Option`] instead of [`MaybeUninit`]
/// is necessary for code safety, as this type returns [`None`] on calling
/// [`Self::get`] if [`Self::init`] has not been run yet, compared to [`UncheckedCell`]
/// where calling [`UncheckedCell::get`] before [`UncheckedCell::init`] is undefined behavior.
///
/// [`MaybeUninit`]: core::mem::MaybeUninit
/// [`UncheckedCell`]: super::unchecked_cell::UncheckedCell
/// [`UncheckedCell::get`]: super::unchecked_cell::UncheckedCell::get
/// [`UncheckedCell::init`]: super::unchecked_cell::UncheckedCell::init
pub struct NicheCell<T>(
    UnsafeCell<Option<T>>
);

/// Safety: this type can be shared across CPUs as the callers have the responsibility to
/// ensure proper data safety because of the `unsafe` contracts on [`init`](Self::init) and [`get`](Self::get)
unsafe impl<T: Send + Sync> Sync for NicheCell<T> {}

impl<T> NicheCell<T> {
    /// Set the inner value
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init(&self, value: T) {
        #[cfg(feature = "debug-checks")]
        {
            use crate::lib::panic::kernel_panic;
            use crate::lib::panic_codes::PanicCode;

            if unsafe { self.get().is_some() } {
                kernel_panic(
                    PanicCode::DoubleInitialization,
                    "Attempted to double-initialize a NicheCell"
                );
            }
        }

        unsafe { *self.0.get() = Some(value) };
    }

    /// Get a reference to the inner value
    ///
    /// # Safety
    /// The caller must guarantee that at this method's entire execution time,
    /// no mutable references or pointers to this data exist or will exist
    pub unsafe fn get(&self) -> &Option<T> {
        unsafe { &*self.0.get() }
    }
}

impl<T> Default for NicheCell<T> {
    /// Constructor; creates a new [`NicheCell`] containing [`None`]
    fn default() -> Self {
        Self(
            UnsafeCell::new(None)
        )
    }
}
