//! logging/_serial.rs
//! Serial Logging on COM1 (stubs, traits, ...)
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

pub trait _Serial {
    fn new(port: SerialPort) -> Self;
    fn init(&self) -> Result<(), &'static str>;
    fn write(&self, string: &str);
}

pub enum SerialPort {
    Serial1,
    Serial2,
    Serial3,
    Serial4
}
