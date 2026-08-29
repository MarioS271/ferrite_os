// SPDX-License-Identifier: GPL-3.0-only
//! x86_64 UART serial port implementation of [`_Serial`].
//!
//! Authors: MarioS271

use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::instructions::port::Port;
use crate::logging::serial::{SerialPort, _Serial};

static COM1_BASE_ADDRESS: u16 = 0x03F8;
static COM2_BASE_ADDRESS: u16 = 0x02F8;
static COM3_BASE_ADDRESS: u16 = 0x03E8;
static COM4_BASE_ADDRESS: u16 = 0x02E8;

/// x86_64 UART serial port implementing [`_Serial`]; do not `write()` before `init()`.
pub struct Serial {
    initialized: AtomicBool,
    /// I/O base address for this COM port (e.g., `0x03F8` for COM1).
    base_addr: u16,
}

impl _Serial for Serial {
    fn new(port: SerialPort) -> Self {
        let base_address = match port {
            SerialPort::Serial1 => { COM1_BASE_ADDRESS }
            SerialPort::Serial2 => { COM2_BASE_ADDRESS }
            SerialPort::Serial3 => { COM3_BASE_ADDRESS }
            SerialPort::Serial4 => { COM4_BASE_ADDRESS }
        };

        Self {
            initialized: AtomicBool::new(false),
            base_addr: base_address,
        }
    }

    fn init(&self) -> Result<(), &'static str> {
        // Safe because:
        // 1) static mut
        //    No threading exists at the time of this being executed, as this function gets called
        //    as one of the first things in kmain
        // 2) at_offset().write()
        //    Safe because only writing to common UART registers (follows the standard 16550 spec,
        //    therefore will not corrupt unrelated things)
        unsafe {
            if self.initialized.load(Ordering::Acquire) {
                return Err("Serial is already initialized");
            }

            self.initialized.store(true, Ordering::Relaxed);

            at_offset(self.base_addr, 1).write(0x00);     // Disable Interrupts.
            at_offset(self.base_addr, 3).write(0x80);     // Enable DLAB (divisor latch access bit)
            at_offset(self.base_addr, 0).write(0x01);     // Baud Rate Divisor (low byte)
            at_offset(self.base_addr, 1).write(0x00);     // Baud Rate Divisor (high byte)
            at_offset(self.base_addr, 3).write(0x03);     // Disable DLAB and configure line
            at_offset(self.base_addr, 2).write(0xC7);     // Enable and clear FIFO
            at_offset(self.base_addr, 4).write(0x0B);     // Set DTR, RTS, OUT2

            Ok(())
        }
    }

    fn write(&self, string: &str) {
        let mut port: Port<u8> = Port::new(self.base_addr);

        for byte in string.bytes() {
            // This calls write_byte which was already declared to be safe because
            // it adheres to 16550 spec
            unsafe {
                // Wait for
                while at_offset(self.base_addr, 5).read() & 0x20 == 0 {
                    core::hint::spin_loop();
                }
                port.write(byte);
            }
        }
    }
}

/// Return a `Port` for the UART register at `offset` from `base_addr`.
fn at_offset(base_addr: u16, offset: u16) -> Port<u8> {
    Port::new(base_addr + offset)
}
