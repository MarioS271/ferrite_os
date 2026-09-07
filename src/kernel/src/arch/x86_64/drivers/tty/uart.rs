// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 UART driver
//!
//! Authors: MarioS271

use crate::drivers::tty::uart::{UartError, UartOps, UartResult};
use core::sync::atomic::{AtomicU8, Ordering};
use x86_64::instructions::port::Port;

/// x86_64 UART driver implementing [`UartOps`]
pub struct Uart {
    state: AtomicU8,
    base_addr: u16
}

impl Uart {
    const INIT_CALLED: u8 = 1 << 0;
    const USABLE: u8 = 1 << 1;

    /// Constructor; creates a new uninitialized [`Uart`] device with the given `base_addr`
    pub fn new(base_addr: u16) -> Self {
        Self {
            state: AtomicU8::default(),
            base_addr,
        }
    }

    /// Wrapper around [`Self::new`]; creates a [`Uart`] device with a `base_addr` of `0x3F8`
    pub fn com1() -> Self {
        Self::new(0x3F8)
    }
    /// Wrapper around [`Self::new`]; creates a [`Uart`] device with a `base_addr` of `0x2F8`
    pub fn com2() -> Self {
        Self::new(0x2F8)
    }
    /// Wrapper around [`Self::new`]; creates a [`Uart`] device with a `base_addr` of `0x3E8`
    pub fn com3() -> Self {
        Self::new(0x3E8)
    }
    /// Wrapper around [`Self::new`]; creates a [`Uart`] device with a `base_addr` of `0x2E8`
    pub fn com4() -> Self {
        Self::new(0x2E8)
    }

    /// Returns the I/O port at `offset` from this UART device's `base_addr`
    fn reg(&self, offset: u16) -> Port<u8> {
        Port::new(self.base_addr + offset)
    }

    /// Checks if the given device is usable (fully initialized)
    #[inline(always)]
    fn is_usable(&self) -> bool {
        self.state.load(Ordering::Acquire) & Self::USABLE == Self::USABLE
    }
}

impl UartOps for Uart {
    fn init(&mut self) -> UartResult<()> {
        let prev = self.state.fetch_or(Self::INIT_CALLED, Ordering::AcqRel);

        if prev & Self::INIT_CALLED != 0 {
            core::hint::cold_path();
            return Err(UartError::AlreadyInitialized);
        }

        // Safety:
        // - writing to standardized x86 UART addresses via Port I/O
        // - writes will only happen once, guaranteed by the self.state guards
        unsafe {
            self.reg(1).write(0x00);     // Disable Interrupts.
            self.reg(3).write(0x80);     // Enable DLAB (divisor latch access bit)
            self.reg(0).write(0x01);     // Baud Rate Divisor (low byte)
            self.reg(1).write(0x00);     // Baud Rate Divisor (high byte)
            self.reg(3).write(0x03);     // Disable DLAB and configure line
            self.reg(2).write(0xC7);     // Enable and clear FIFO
            self.reg(4).write(0x0B);     // Set DTR, RTS, OUT2
        }

        self.state.fetch_or(Self::USABLE, Ordering::AcqRel);
        Ok(())
    }

    fn read_byte(&mut self) -> UartResult<u8> {
        if !self.is_usable() {
            core::hint::cold_path();
            return Err(UartError::NotInitialized);
        }

        // Safety: this is a standardized x86 UART operation (spin until byte is avail, then read)
        unsafe {
            while self.reg(5).read() & 0x01 == 0 {
                core::hint::spin_loop();
            }
            Ok(self.reg(0).read())
        }
    }

    fn write_byte(&mut self, byte: u8) -> UartResult<()> {
        if !self.is_usable() {
            core::hint::cold_path();
            return Err(UartError::NotInitialized);
        }

        // Safety: this is a standardized x86 UART operation (spin until transmitter is ready, then write)
        unsafe {
            while self.reg(5).read() & 0x20 == 0 {
                core::hint::spin_loop();
            }
            self.reg(0).write(byte);
        }

        Ok(())
    }
}
