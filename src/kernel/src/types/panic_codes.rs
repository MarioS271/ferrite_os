//! types/panic_codes.rs
//! Panic Codes used to differentiate different kinds of kernel panics
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

#[repr(u16)]
pub enum PanicCode {
    // General
    Unknown = 0x0000,
    ManuallyTriggeredPanic = 0x0001,

    // Exceptions
    DoubleFault = 0x0100,
    GeneralProtectionFault = 0x0101,
    PageFault = 0x102,

    // Memory
    NoValidMemMapEntry = 0x0300,
    PmmNotInitialized = 0x0301,
    DoubleFree = 0x0302,
    IllegalFree = 0x0303,

    // Display
    InvalidPsf2MagicNumber = 0x1000,
}

impl PanicCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            // General
            PanicCode::Unknown => "Unknown",
            PanicCode::ManuallyTriggeredPanic => "ManuallyTriggeredPanic",

            // Exceptions
            PanicCode::DoubleFault => "DoubleFault",
            PanicCode::GeneralProtectionFault => "GeneralProtectionFault",
            PanicCode::PageFault => "PageFault",

            // Memory
            PanicCode::NoValidMemMapEntry => "NoValidMemMapEntry",
            PanicCode::PmmNotInitialized => "PmmNotInitialized",
            PanicCode::DoubleFree => "DoubleFree",
            PanicCode::IllegalFree => "IllegalFree",

            // Display
            PanicCode::InvalidPsf2MagicNumber => "InvalidPsf2MagicNumber"
        }
    }
}