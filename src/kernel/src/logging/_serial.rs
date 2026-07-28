//! Serial port abstraction: trait definition and port identifier enum.
//!
//! Separating the trait from the x86_64 implementation lets the arch module re-export
//! the concrete type under a common path while keeping the trait as the stable interface.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

/// Interface for a serial port implementation.
///
/// Implementations are arch-specific; the x86_64 implementation is in
/// `logging::x86_64::serial`.
pub trait _Serial {
    /// Create a new (uninitialized) serial port instance for the given COM port.
    fn new(port: SerialPort) -> Self;

    /// Program the UART hardware and mark the port as ready to use.
    ///
    /// Returns `Err` if the port is already initialized; `Ok` otherwise.
    fn init(&self) -> Result<(), &'static str>;

    /// Write all bytes of `string` to the serial port, blocking until each byte
    /// is accepted by the transmit FIFO.
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
