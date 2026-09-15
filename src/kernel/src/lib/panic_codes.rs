// SPDX-License-Identifier: GPL-3.0-only
//! Panic codes for classifying kernel panics
//!
//! Authors: MarioS271

/// Declares [`PanicCode`] together with its accessors from a single list
macro_rules! panic_codes {
    ($( $(#[$attr:meta])* $name:ident = $value:expr, $kind:expr; )*) => {
        #[repr(u16)]
        pub enum PanicCode {
            $( $(#[$attr])* $name = $value, )*
        }

        impl PanicCode {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( $(#[$attr])* PanicCode::$name => stringify!($name), )*
                }
            }

            pub fn get_error_type_str(&self) -> &'static str {
                match self {
                    $( $(#[$attr])* PanicCode::$name => $kind, )*
                }
            }
        }
    };
}

panic_codes! {
    // General
    Unknown = 0x0000, "Unknown";
    ManuallyTriggeredPanic = 0x0001, "IntendedPanic";
    InitFailure = 0x0002, "RuntimeError";

    // Programmer Failures
    UninitializedAccess = 0x0003, "ProgrammerError";
    DoubleInitialization = 0x0004, "ProgrammerError";

    // x86_64 Exceptions
    #[cfg(target_arch = "x86_64")] IllegalInterrupt = 0x0099, "CpuException";
    #[cfg(target_arch = "x86_64")] DivideError = 0x0100, "CpuException";
    #[cfg(target_arch = "x86_64")] NmiHardwareFailure = 0x0102, "CpuException";
    #[cfg(target_arch = "x86_64")] Overflow = 0x0104, "CpuException";
    #[cfg(target_arch = "x86_64")] InvalidOpcode = 0x0106, "CpuException";
    #[cfg(target_arch = "x86_64")] DeviceNotAvail = 0x0107, "CpuException";
    #[cfg(target_arch = "x86_64")] DoubleFault = 0x0108, "CpuException";
    #[cfg(target_arch = "x86_64")] InvalidTss = 0x0110, "CpuException";
    #[cfg(target_arch = "x86_64")] SegmentNotPresent = 0x0111, "CpuException";
    #[cfg(target_arch = "x86_64")] StackSegmentFault = 0x0112, "CpuException";
    #[cfg(target_arch = "x86_64")] GeneralProtectionFault = 0x0113, "CpuException";
    #[cfg(target_arch = "x86_64")] PageFault = 0x0114, "CpuException";
    #[cfg(target_arch = "x86_64")] X87FloatingPoint = 0x0116, "CpuException";
    #[cfg(target_arch = "x86_64")] AlignmentCheck = 0x0117, "CpuException";
    #[cfg(target_arch = "x86_64")] MachineCheck = 0x0118, "CpuException";
    #[cfg(target_arch = "x86_64")] SimdFloatingPoint = 0x0119, "CpuException";
    #[cfg(target_arch = "x86_64")] Virtualization = 0x0120, "CpuException";
    #[cfg(target_arch = "x86_64")] VmmCommunicationException = 0x0129, "CpuException";
    #[cfg(target_arch = "x86_64")] SecurityException = 0x0130, "CpuException";

    // Memory
    NoValidMemMapEntry = 0x0300, "RuntimeError";
    DoubleFree = 0x0301, "ProgrammerError";
    IllegalFree = 0x0302, "ProgrammerError";
    OutOfMemory = 0x0303, "RuntimeError";
    MemoryMappingCollision = 0x0304, "ProgrammerError";
    MisalignedAddress = 0x0305, "ProgrammerError";
    // Memory: Paging
    InvalidPageOperation = 0x0320, "RuntimeError";

    // Binaries
    InvalidBinary = 0x0400, "RuntimeError";

    // Display
    MalformedPsf2Font = 0x1000, "RuntimeError / ProgrammerError";
}
