// SPDX-License-Identifier: GPL-3.0-only
//! Simple State struct; used in early init and on a kernel panic
//!
//! Authors: MarioS271

use crate::logging::serial::{Serial};
use crate::screen::basic::font::Psf2Font;
use crate::screen::basic::framebuffer::BasicFramebuffer;
use crate::types::irq_mutex::IrqMutex;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

/// Data structure for keeping the serial logger and basic fb resources for early boot and panic
pub(crate) static SIMPLE_STATE: SimpleKernelState = SimpleKernelState {
    serial: UnsafeCell::new(MaybeUninit::uninit()),
    basic_fb: UnsafeCell::new(MaybeUninit::uninit()),
    basic_fb_psf2_font: UnsafeCell::new(MaybeUninit::uninit()),
};

pub struct SimpleKernelState {
    serial: UnsafeCell<MaybeUninit<IrqMutex<Serial>>>,
    basic_fb: UnsafeCell<MaybeUninit<IrqMutex<BasicFramebuffer>>>,
    basic_fb_psf2_font: UnsafeCell<MaybeUninit<Psf2Font>>,
}

unsafe impl Sync for SimpleKernelState {}

impl SimpleKernelState {
    /// Move the given [`Serial`] into `SIMPLE_STATE::serial`
    pub fn init_serial(&self, serial: Serial) {
        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.serial.get()).write(IrqMutex::new(serial)); }
    }

    /// Move the given [`BasicFramebuffer`] into `SIMPLE_STATE::basic_fb`
    pub fn init_basic_fb(&self, basic_fb: BasicFramebuffer) {
        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.basic_fb.get()).write(IrqMutex::new(basic_fb)); }
    }

    /// Move the given [`Psf2Font`] into `SIMPLE_STATE::basic_fb_psf2_font`
    pub fn init_basic_fb_psf2_font(&self, basic_fb_psf2_font: Psf2Font) {
        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.basic_fb_psf2_font.get()).write(basic_fb_psf2_font); }
    }

    /// Getter for SIMPLE_STATE::serial
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    pub fn serial(&self) -> &IrqMutex<Serial> {
        // Safety:
        // 1) the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        // 2) assume_init_ref is not guaranteed to be safe, the caller must guarantee this
        unsafe { (*self.serial.get()).assume_init_ref() }
    }

    /// Getter for SIMPLE_STATE::basic_fb
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    pub fn basic_fb(&self) -> &IrqMutex<BasicFramebuffer> {
        // Safety:
        // 1) the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        // 2) assume_init_ref is not guaranteed to be safe, the caller must guarantee this
        unsafe { (*self.basic_fb.get()).assume_init_ref() }
    }

    /// Getter for SIMPLE_STATE::basic_fb_psf2_font
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    pub fn basic_fb_psf2_font(&self) -> &Psf2Font {
        // Safety:
        // 1) the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        // 2) assume_init_ref is not guaranteed to be safe, the caller must guarantee this
        unsafe { (*self.basic_fb_psf2_font.get()).assume_init_ref() }
    }
}
