// SPDX-License-Identifier: GPL-3.0-only
//! Simple State struct; used in early init and on a kernel panic
//!
//! Authors: MarioS271

use crate::logging::serial::Serial;
use crate::panic::kernel_panic;
use crate::screen::basic::font::Psf2Font;
use crate::screen::basic::framebuffer::BasicFramebuffer;
use crate::types::irq_mutex::IrqMutex;
use crate::types::panic_codes::PanicCode;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU8, Ordering};

/// Data structure for keeping the serial logger and basic fb resources for early boot and panic
pub(crate) static SIMPLE_STATE: SimpleKernelState = SimpleKernelState {
    is_init: AtomicU8::new(0),
    serial: UnsafeCell::new(MaybeUninit::uninit()),
    basic_fb: UnsafeCell::new(MaybeUninit::uninit()),
    basic_fb_psf2_font: UnsafeCell::new(MaybeUninit::uninit()),
};

pub struct SimpleKernelState {
    is_init: AtomicU8,
    serial: UnsafeCell<MaybeUninit<IrqMutex<Serial>>>,
    basic_fb: UnsafeCell<MaybeUninit<IrqMutex<BasicFramebuffer>>>,
    basic_fb_psf2_font: UnsafeCell<MaybeUninit<Psf2Font>>,
}

/// Safety: these values can only be initialized once (guarded by the `SIMPLE_STATE::is_init` atomic)
/// and the ones which will later be mutated are wrapped in an [`IrqMutex`]
unsafe impl Sync for SimpleKernelState {}

impl SimpleKernelState {
    /// Move the given [`Serial`] into `SIMPLE_STATE::serial`
    pub fn init_serial(&self, serial: Serial) {
        if self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsSerialInit as u8 != 0 {
            core::hint::cold_path();
            return;
        }

        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.serial.get()).write(IrqMutex::new(serial)); }
        self.is_init.fetch_or(SimpleStateInit::IsSerialInit as u8, Ordering::AcqRel);
    }

    /// Move the given [`BasicFramebuffer`] into `SIMPLE_STATE::basic_fb`
    pub fn init_basic_fb(&self, basic_fb: BasicFramebuffer) {
        if self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsBasicFbInit as u8 != 0 {
            core::hint::cold_path();
            return;
        }

        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.basic_fb.get()).write(IrqMutex::new(basic_fb)); }
        self.is_init.fetch_or(SimpleStateInit::IsBasicFbInit as u8, Ordering::AcqRel);
    }

    /// Move the given [`Psf2Font`] into `SIMPLE_STATE::basic_fb_psf2_font`
    pub fn init_basic_fb_psf2_font(&self, basic_fb_psf2_font: Psf2Font) {
        if self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsBasicFbFontInit as u8 != 0 {
            core::hint::cold_path();
            return;
        }

        // Safety: the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        unsafe { (*self.basic_fb_psf2_font.get()).write(basic_fb_psf2_font); }
        self.is_init.fetch_or(SimpleStateInit::IsBasicFbFontInit as u8, Ordering::AcqRel);
    }

    /// Getter for SIMPLE_STATE::serial
    ///
    /// # Safety
    /// This getter wraps the unsafe function [`MaybeUninit::assume_init_ref()`] in a safe getter
    /// to avoid many unsafe blocks everywhere. This does NOT remove the unsafe factor, the caller
    /// must still ensure that this value is initialized BEFORE the getter is called. Otherwise,
    /// undefined data will be returned.
    ///
    /// Additionally, to avoid data races, this should not be called when multiple cores are active.
    pub fn serial(&self) -> &IrqMutex<Serial> {
        if self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsSerialInit as u8 == 0 {
            kernel_panic(
                PanicCode::UninitializedAccess,
                "Cannot access SIMPLE_STATE::serial before it is initialized"
            );
        }

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
    ///
    /// Additionally, to avoid data races, this should not be called when multiple cores are active.
    pub fn basic_fb(&self) -> &IrqMutex<BasicFramebuffer> {
        if self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsBasicFbInit as u8 == 0 {
            kernel_panic(
                PanicCode::UninitializedAccess,
                "Cannot access SIMPLE_STATE::basic_fb before it is initialized"
            );
        }

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
    ///
    /// Additionally, to avoid data races, this should not be called when multiple cores are active.
    pub fn basic_fb_psf2_font(&self) -> &Psf2Font {
        if self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsBasicFbFontInit as u8 == 0 {
            kernel_panic(
                PanicCode::UninitializedAccess,
                "Cannot access SIMPLE_STATE::basic_fb_psf2_font before it is initialized"
            );
        }

        // Safety:
        // 1) the deref is always safe, as the deref'd object is the statically initialized MaybeUninit
        // 2) assume_init_ref is not guaranteed to be safe, the caller must guarantee this
        unsafe { (*self.basic_fb_psf2_font.get()).assume_init_ref() }
    }

    /// Check whether `SIMPLE_STATE::serial` is initialized
    #[inline]
    pub fn is_serial_initialized(&self) -> bool {
        self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsSerialInit as u8 != 0
    }

    /// Check whether `SIMPLE_STATE::basic_fb` is initialized
    #[inline]
    pub fn is_basic_fb_initialized(&self) -> bool {
        self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsBasicFbInit as u8 != 0
    }

    /// Check whether `SIMPLE_STATE::basic_fb_psf2_font` is initialized
    #[inline]
    pub fn is_basic_fb_psf2_font_initialized(&self) -> bool {
        self.is_init.load(Ordering::Acquire) & SimpleStateInit::IsBasicFbFontInit as u8 != 0
    }
}

/// Enum helper to not have to write the raw bitshifts on every is_init check
#[repr(u8)]
enum SimpleStateInit {
    IsSerialInit = 1 << 0,
    IsBasicFbInit = 1 << 1,
    IsBasicFbFontInit = 1 << 2
}
