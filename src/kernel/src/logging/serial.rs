// SPDX-License-Identifier: GPL-3.0-only
//! Serial port abstraction: the [`_Serial`] trait and the [`SerialPort`] identifier enum.
//!
//! Authors: MarioS271

pub use super::x86_64::serial::*;

/// Interface for a serial port implementation.
pub trait _Serial {
    /// Create a new (uninitialized) serial port instance for the given COM port.
    fn new(port: SerialPort) -> Self;

    /// Program the UART hardware and mark the port ready; `Err` if already initialized.
    fn init(&self) -> Result<(), &'static str>;

    /// Write all bytes of `string` to the port, blocking until each is accepted.
    fn write(&self, string: &str);
}

/// Identifier for one of the four standard IBM PC serial ports.
pub enum SerialPort {
    /// COM1 — I/O base address `0x03F8`, typically connected to the host in QEMU.
    Serial1,
    /// COM2 — I/O base address `0x02F8`.
    Serial2,
    /// COM3 — I/O base address `0x03E8`.
    Serial3,
    /// COM4 — I/O base address `0x02E8`.
    Serial4
}
