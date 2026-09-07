// SPDX-License-Identifier: GPL-3.0-only
//! Simple State struct; used in early boot and on a kernel panic
//!
//! Authors: MarioS271

use crate::drivers::gpu::basic_fb::font::Psf2Font;
use crate::drivers::gpu::basic_fb::framebuffer::BasicFramebuffer;
use crate::drivers::tty::uart::Uart;
use crate::lib::panic::kernel_panic;
use crate::lib::panic_codes::PanicCode;
use crate::lib::sync::irq_mutex::IrqMutex;
use crate::lib::sync::unchecked_cell::UncheckedCell;
use crate::lib::types::boot_info::PanicAction;
use core::sync::atomic::{AtomicU8, Ordering};

/// Data structure for keeping the serial UART logger and basic_fb fb resources for early boot and panic
/// The default initializations should not contain any non-null value in order for [`SIMPLE_STATE`]
/// to be able to live in `.bss` to reduce binary size
pub(crate) static SIMPLE_STATE: SimpleKernelState = SimpleKernelState {
    is_init: AtomicU8::new(0),
    uart: UncheckedCell::new(),
    basic_fb: UncheckedCell::new(),
    basic_fb_psf2_font: UncheckedCell::new(),
    panic_action: AtomicU8::new(0)
};

#[repr(align(64))]
pub struct SimpleKernelState {
    is_init: AtomicU8,
    uart: UncheckedCell<IrqMutex<Uart>>,
    basic_fb: UncheckedCell<IrqMutex<BasicFramebuffer>>,
    basic_fb_psf2_font: UncheckedCell<Psf2Font>,
    panic_action: AtomicU8
}

impl SimpleKernelState {
    const IS_UART_INIT: u8 = 1 << 0;
    const IS_BASIC_FB_INIT: u8 = 1 << 1;
    const IS_BASIC_FB_FONT_INIT: u8 = 1 << 2;

    /// Move the given [`Uart`] into `SIMPLE_STATE::uart`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_uart(&self, uart: Uart) {
        if self.is_init.load(Ordering::Acquire) & Self::IS_UART_INIT != 0 {
            kernel_panic(
                PanicCode::DoubleInitialization,
                "Attempted to double-initialize SIMPLE_STATE.uart"
            );
        }

        unsafe { self.uart.init(IrqMutex::new(uart)) };
        self.is_init.fetch_or(Self::IS_UART_INIT, Ordering::Release);
    }

    /// Move the given [`BasicFramebuffer`] into `SIMPLE_STATE::basic_fb`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_basic_fb(&self, basic_fb: BasicFramebuffer) {
        if self.is_init.load(Ordering::Acquire) & Self::IS_BASIC_FB_INIT != 0 {
            kernel_panic(
                PanicCode::DoubleInitialization,
                "Attempted to double-initialize SIMPLE_STATE.basic_fb"
            );
        }

        unsafe { self.basic_fb.init(IrqMutex::new(basic_fb)) };
        self.is_init.fetch_or(Self::IS_BASIC_FB_INIT, Ordering::Release);
    }

    /// Move the given [`Psf2Font`] into `SIMPLE_STATE::basic_fb_psf2_font`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this method has never been called before and will never be called again
    /// - That at the time of calling this method, no references or pointers to this data exist
    /// - While this method is being called, no other CPU is working with the given data
    pub unsafe fn init_basic_fb_psf2_font(&self, basic_fb_psf2_font: Psf2Font) {
        if self.is_init.load(Ordering::Acquire) & Self::IS_BASIC_FB_FONT_INIT != 0 {
            kernel_panic(
                PanicCode::DoubleInitialization,
                "Attempted to double-initialize SIMPLE_STATE.basic_fb_psf2_font"
            );
        }

        unsafe { self.basic_fb_psf2_font.init(basic_fb_psf2_font) };
        self.is_init.fetch_or(Self::IS_BASIC_FB_FONT_INIT, Ordering::Release);
    }

    /// Move the given [`PanicAction`] into `SIMPLE_STATE::panic_action`
    pub fn set_panic_action(&self, panic_action: PanicAction) {
        self.panic_action.store(panic_action as u8, Ordering::Release);
    }

    /// Getter for `SIMPLE_STATE::uart`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn uart(&self) -> &IrqMutex<Uart> {
        unsafe { self.uart.get() }
    }

    /// Getter for `SIMPLE_STATE::basic_fb`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn basic_fb(&self) -> &IrqMutex<BasicFramebuffer> {
        unsafe { self.basic_fb.get() }
    }

    /// Getter for `SIMPLE_STATE::basic_fb_psf2_font`
    ///
    /// # Safety
    /// The caller must guarantee the following:
    /// - That this value's `init` method has already been called before
    /// - That at this method's entire execution time, no mutable references or pointers to this data
    ///   exist or will exist
    pub unsafe fn basic_fb_psf2_font(&self) -> &Psf2Font {
        unsafe { self.basic_fb_psf2_font.get() }
    }

    /// Getter for `SIMPLE_STATE::panic_action`
    pub fn panic_action(&self) -> PanicAction {
        PanicAction::from_u8(self.panic_action.load(Ordering::Acquire)).unwrap_or(crate::config::DEFAULT_PANIC_ACTION)
    }

    /// Check whether `SIMPLE_STATE::uart` is initialized
    pub fn is_uart_initialized(&self) -> bool {
        self.is_init.load(Ordering::Acquire) & Self::IS_UART_INIT != 0
    }

    /// Check whether `SIMPLE_STATE::basic_fb` is initialized
    pub fn is_basic_fb_initialized(&self) -> bool {
        self.is_init.load(Ordering::Acquire) & Self::IS_BASIC_FB_INIT != 0
    }

    /// Check whether `SIMPLE_STATE::basic_fb_psf2_font` is initialized
    pub fn is_basic_fb_psf2_font_initialized(&self) -> bool {
        self.is_init.load(Ordering::Acquire) & Self::IS_BASIC_FB_FONT_INIT != 0
    }
}
