// SPDX-License-Identifier: GPL-3.0-only
//! Panic codes for classifying kernel panics
//!
//! Authors: MarioS271

/// Enum which represents different possible kernel panic scenarios
#[repr(u16)]
pub enum PanicCode {
    // General
    Unknown = 0x0000,
    ManuallyTriggeredPanic = 0x0001,
    InitFailure = 0x0002,
    UninitializedAccess = 0x0003,

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
    DoubleFree = 0x0301,
    IllegalFree = 0x0302,
    OutOfMemory = 0x0303,
    // Memory: Paging
    InvalidPageOperation = 0x320,

    // Display
    InvalidPsf2MagicNumber = 0x1000,
}

impl PanicCode {
    /// Return the variant name as a static string, suitable for printing in a panic message.
    pub fn as_str(&self) -> &'static str {
        use PanicCode::*;
        match self {
            // General
            Unknown => "Unknown",
            ManuallyTriggeredPanic => "ManuallyTriggeredPanic",
            InitFailure => "InitFailure",
            UninitializedAccess => "UninitializedAccess",

            // Exceptions
            IllegalInterrupt => "IllegalInterrupt",
            DivideError => "DivideError",
            NmiHardwareFailiure => "NmiHardwareFailiure",
            Overflow => "Overflow",
            InvalidOpcode => "InvalidOpcode",
            DeviceNotAvail => "DeviceNotAvail",
            DoubleFault => "DoubleFault",
            InvalidTss => "InvalidTss",
            SegmentNotPresent => "SegmentNotPresent",
            StackSegmentFault => "StackSegmentFault",
            GeneralProtectionFault => "GeneralProtectionFault",
            PageFault => "PageFault",
            X87FloatingPoint => "X87FloatingPoint",
            AlignmentCheck => "AlignmentCheck",
            MachineCheck => "MachineCheck",
            SimdFloatingPoint => "SimdFloatingPoint",
            Virtualization => "Virtualization",
            VmmCommunicationException => "VmmCommunicationException",
            SecurityException => "SecurityException",

            // Memory
            NoValidMemMapEntry => "NoValidMemMapEntry",
            DoubleFree => "DoubleFree",
            IllegalFree => "IllegalFree",
            OutOfMemory => "OutOfMemory",
            // Memory: Paging
            InvalidPageOperation => "InvalidPageOperation",

            // Display
            InvalidPsf2MagicNumber => "InvalidPsf2MagicNumber",
        }
    }

    /// Returns a string which tell you what general type of panic this was
    /// (programmer error, runtime error, ...)
    pub fn get_error_type_str(&self) -> &'static str {
        use PanicCode::*;
        match self {
            Unknown => "Unknown",
            ManuallyTriggeredPanic => "IntendedPanic",

            DoubleFree | IllegalFree | UninitializedAccess => "ProgrammerError",

            InitFailure | NoValidMemMapEntry | OutOfMemory
            | InvalidPageOperation => "RuntimeError",

            IllegalInterrupt | DivideError | NmiHardwareFailiure | Overflow
            | InvalidOpcode | DeviceNotAvail | DoubleFault | InvalidTss
            | SegmentNotPresent | StackSegmentFault | GeneralProtectionFault
            | PageFault | X87FloatingPoint | AlignmentCheck | MachineCheck
            | SimdFloatingPoint | Virtualization | VmmCommunicationException
            | SecurityException => "CpuException",

            InvalidPsf2MagicNumber => "RuntimeError / ProgrammerError"
        }
    }
}
