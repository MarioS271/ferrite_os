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

    // Programmer Failiures
    UninitializedAccess = 0x0003,
    DoubleInitialization = 0x0004,

    // x86_64 Exceptions
    #[cfg(target_arch = "x86_64")] IllegalInterrupt = 0x0099,
    #[cfg(target_arch = "x86_64")] DivideError = 0x0100,
    #[cfg(target_arch = "x86_64")] NmiHardwareFailiure = 0x0102,
    #[cfg(target_arch = "x86_64")] Overflow = 0x0104,
    #[cfg(target_arch = "x86_64")] InvalidOpcode = 0x0106,
    #[cfg(target_arch = "x86_64")] DeviceNotAvail = 0x0107,
    #[cfg(target_arch = "x86_64")] DoubleFault = 0x0108,
    #[cfg(target_arch = "x86_64")] InvalidTss = 0x0110,
    #[cfg(target_arch = "x86_64")] SegmentNotPresent = 0x0111,
    #[cfg(target_arch = "x86_64")] StackSegmentFault = 0x0112,
    #[cfg(target_arch = "x86_64")] GeneralProtectionFault = 0x0113,
    #[cfg(target_arch = "x86_64")] PageFault = 0x0114,
    #[cfg(target_arch = "x86_64")] X87FloatingPoint = 0x0116,
    #[cfg(target_arch = "x86_64")] AlignmentCheck = 0x0117,
    #[cfg(target_arch = "x86_64")] MachineCheck = 0x0118,
    #[cfg(target_arch = "x86_64")] SimdFloatingPoint = 0x0119,
    #[cfg(target_arch = "x86_64")] Virtualization = 0x0120,
    #[cfg(target_arch = "x86_64")] VmmCommunicationException = 0x0129,
    #[cfg(target_arch = "x86_64")] SecurityException = 0x0130,

    // Memory
    NoValidMemMapEntry = 0x0300,
    DoubleFree = 0x0301,
    IllegalFree = 0x0302,
    OutOfMemory = 0x0303,
    MisalignedAddress = 0x0304,
    // Memory: Paging
    InvalidPageOperation = 0x320,

    // Display
    MalformedPsf2Font = 0x1000,
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

            // Programmer Failiure
            UninitializedAccess => "UninitializedAccess",
            DoubleInitialization => "DoubleInitialization",

            // x86_64 Exceptions
            #[cfg(target_arch = "x86_64")] IllegalInterrupt => "IllegalInterrupt",
            #[cfg(target_arch = "x86_64")] DivideError => "DivideError",
            #[cfg(target_arch = "x86_64")] NmiHardwareFailiure => "NmiHardwareFailiure",
            #[cfg(target_arch = "x86_64")] Overflow => "Overflow",
            #[cfg(target_arch = "x86_64")] InvalidOpcode => "InvalidOpcode",
            #[cfg(target_arch = "x86_64")] DeviceNotAvail => "DeviceNotAvail",
            #[cfg(target_arch = "x86_64")] DoubleFault => "DoubleFault",
            #[cfg(target_arch = "x86_64")] InvalidTss => "InvalidTss",
            #[cfg(target_arch = "x86_64")] SegmentNotPresent => "SegmentNotPresent",
            #[cfg(target_arch = "x86_64")] StackSegmentFault => "StackSegmentFault",
            #[cfg(target_arch = "x86_64")] GeneralProtectionFault => "GeneralProtectionFault",
            #[cfg(target_arch = "x86_64")] PageFault => "PageFault",
            #[cfg(target_arch = "x86_64")] X87FloatingPoint => "X87FloatingPoint",
            #[cfg(target_arch = "x86_64")] AlignmentCheck => "AlignmentCheck",
            #[cfg(target_arch = "x86_64")] MachineCheck => "MachineCheck",
            #[cfg(target_arch = "x86_64")] SimdFloatingPoint => "SimdFloatingPoint",
            #[cfg(target_arch = "x86_64")] Virtualization => "Virtualization",
            #[cfg(target_arch = "x86_64")] VmmCommunicationException => "VmmCommunicationException",
            #[cfg(target_arch = "x86_64")] SecurityException => "SecurityException",

            // Memory
            NoValidMemMapEntry => "NoValidMemMapEntry",
            DoubleFree => "DoubleFree",
            IllegalFree => "IllegalFree",
            OutOfMemory => "OutOfMemory",
            MisalignedAddress => "MisalignedAddress",
            // Memory: Paging
            InvalidPageOperation => "InvalidPageOperation",

            // Display
            MalformedPsf2Font => "MalformedPsf2Font",
        }
    }

    /// Returns a string which tell you what general type of panic this was
    /// (programmer error, runtime error, ...)
    pub fn get_error_type_str(&self) -> &'static str {
        use PanicCode::*;
        match self {
            Unknown => "Unknown",
            ManuallyTriggeredPanic => "IntendedPanic",

            DoubleFree | IllegalFree | UninitializedAccess
            | DoubleInitialization | MisalignedAddress => "ProgrammerError",

            InitFailure | NoValidMemMapEntry | OutOfMemory
            | InvalidPageOperation => "RuntimeError",

            #[cfg(target_arch = "x86_64")] IllegalInterrupt => "CpuException",
            #[cfg(target_arch = "x86_64")] DivideError => "CpuException",
            #[cfg(target_arch = "x86_64")] NmiHardwareFailiure => "CpuException",
            #[cfg(target_arch = "x86_64")] Overflow => "CpuException",
            #[cfg(target_arch = "x86_64")] InvalidOpcode => "CpuException",
            #[cfg(target_arch = "x86_64")] DeviceNotAvail => "CpuException",
            #[cfg(target_arch = "x86_64")] DoubleFault => "CpuException",
            #[cfg(target_arch = "x86_64")] InvalidTss => "CpuException",
            #[cfg(target_arch = "x86_64")] SegmentNotPresent => "CpuException",
            #[cfg(target_arch = "x86_64")] StackSegmentFault => "CpuException",
            #[cfg(target_arch = "x86_64")] GeneralProtectionFault => "CpuException",
            #[cfg(target_arch = "x86_64")] PageFault => "CpuException",
            #[cfg(target_arch = "x86_64")] X87FloatingPoint => "CpuException",
            #[cfg(target_arch = "x86_64")] AlignmentCheck => "CpuException",
            #[cfg(target_arch = "x86_64")] MachineCheck => "CpuException",
            #[cfg(target_arch = "x86_64")] SimdFloatingPoint => "CpuException",
            #[cfg(target_arch = "x86_64")] Virtualization => "CpuException",
            #[cfg(target_arch = "x86_64")] VmmCommunicationException => "CpuException",
            #[cfg(target_arch = "x86_64")] SecurityException => "CpuException",

            MalformedPsf2Font => "RuntimeError / ProgrammerError"
        }
    }
}
