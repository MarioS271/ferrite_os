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
    IllegalInterrupt = 0x0099,
    DivideError = 0x0100,
    NmiHardwareFailiure = 0x0102,
    Overflow = 0x0104,
    InvalidOpcode = 0x0106,
    DeviceNotAvail = 0x0107,
    DoubleFault = 0x0108,
    InvalidTss = 0x0110,
    SegmentNotPresent = 0x0111,
    StackSegmentFault = 0x0112,
    GeneralProtectionFault = 0x0113,
    PageFault = 0x0114,
    X87FloatingPoint = 0x0116,
    AlignmentCheck = 0x0117,
    MachineCheck = 0x0118,
    SimdFloatingPoint = 0x0119,
    Virtualization = 0x0120,
    VmmCommunicationException = 0x0129,
    SecurityException = 0x0130,

    // Memory
    NoValidMemMapEntry = 0x0300,
    PmmNotInitialized = 0x0301,
    DoubleFree = 0x0302,
    IllegalFree = 0x0303,
    OutOfMemory = 0x0304,
    // Memory: Paging
    InvalidPageOperation = 0x320,

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
            PanicCode::IllegalInterrupt => "IllegalInterrupt",
            PanicCode::DivideError => "DivideError",
            PanicCode::NmiHardwareFailiure => "NmiHardwareFailiure",
            PanicCode::Overflow => "Overflow",
            PanicCode::InvalidOpcode => "InvalidOpcode",
            PanicCode::DeviceNotAvail => "DeviceNotAvail",
            PanicCode::DoubleFault => "DoubleFault",
            PanicCode::InvalidTss => "InvalidTss",
            PanicCode::SegmentNotPresent => "SegmentNotPresent",
            PanicCode::StackSegmentFault => "StackSegmentFault",
            PanicCode::GeneralProtectionFault => "GeneralProtectionFault",
            PanicCode::PageFault => "PageFault",
            PanicCode::X87FloatingPoint => "X87FloatingPoint",
            PanicCode::AlignmentCheck => "AlignmentCheck",
            PanicCode::MachineCheck => "MachineCheck",
            PanicCode::SimdFloatingPoint => "SimdFloatingPoint",
            PanicCode::Virtualization => "Virtualization",
            PanicCode::VmmCommunicationException => "VmmCommunicationException",
            PanicCode::SecurityException => "SecurityException",

            // Memory
            PanicCode::NoValidMemMapEntry => "NoValidMemMapEntry",
            PanicCode::PmmNotInitialized => "PmmNotInitialized",
            PanicCode::DoubleFree => "DoubleFree",
            PanicCode::IllegalFree => "IllegalFree",
            PanicCode::OutOfMemory => "OutOfMemory",
            // Memory: Paging
            PanicCode::InvalidPageOperation => "InvalidPageOperation",

            // Display
            PanicCode::InvalidPsf2MagicNumber => "InvalidPsf2MagicNumber"
        }
    }
}
