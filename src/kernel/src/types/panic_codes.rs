//! panic_codes.rs
//! Panic Codes used to differentiate different kinds of kernel panics
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

#[repr(u16)]
pub enum PanicCode {
    Unknown = 0x0000,
    ManuallyTriggeredPanic = 0x0001,
    InvalidPsf2MagicNumber = 0x1000,
}

impl PanicCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PanicCode::Unknown => "Unknown",
            PanicCode::ManuallyTriggeredPanic => "ManuallyTriggeredPanic",
            PanicCode::InvalidPsf2MagicNumber => "InvalidPsf2MagicNumber"
        }
    }
}