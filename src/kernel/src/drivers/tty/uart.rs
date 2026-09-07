// SPDX-License-Identifier: GPL-3.0-only
//! Serial port abstraction: the [`_Serial`] trait and the [`SerialPort`] identifier enums.
//!
//! Authors: MarioS271

#[cfg(target_arch = "x86_64")]
pub use crate::arch::x86_64::drivers::tty::uart::Uart;

/// Trait which defines standardized UART operations across different serial implementations
pub trait UartOps {
    /// Initialize the UART device
    fn init(&mut self) -> UartResult<()>;

    /// Read one byte from the UART device
    fn read_byte(&mut self) -> UartResult<u8>;

    /// Write one byte to the UART device
    fn write_byte(&mut self, byte: u8) -> UartResult<()>;

    /// Write a string to the serial device
    /// Default implementation calls [`Self::write_byte`] for each byte in the string
    fn write_string(&mut self, string: &str) -> UartResult<()> {
        for byte in string.bytes() {
            self.write_byte(byte)?;
        }
        Ok(())
    }
}

pub type UartResult<T> = Result<T, UartError>;
pub enum UartError {
    AlreadyInitialized,
    NotInitialized
}
impl UartError {
    /// Return the name of `self` as a string slice
    pub fn as_str(&self) -> &str {
        use UartError::*;
        match self {
            AlreadyInitialized => "AlreadyInitialized",
            NotInitialized => "NotInitialized"
        }
    }
}
