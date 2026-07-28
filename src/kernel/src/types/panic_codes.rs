// SPDX-License-Identifier: GPL-3.0-only
//! Panic codes for classifying kernel panics.
//!
//! Authors: MarioS271

/// Numeric classification of the reason for a kernel panic.
///
/// Each variant has a `u16` discriminant. Codes are grouped by category:
/// - `0x0000–0x00FF`: general / catch-all
/// - `0x0100–0x01FF`: CPU exception handlers
/// - `0x0300–0x03FF`: memory subsystem
/// - `0x1000–0x1FFF`: display / font subsystem
///
/// Call [`as_str`](PanicCode::as_str) to get a human-readable name for any variant.
#[repr(u16)]
pub enum PanicCode {
    // General
    Unknown = 0x0000,
    ManuallyTriggeredPanic = 0x0001,
    /// A subsystem failed to initialize during boot.
    InitFailure = 0x0002,

    // Exceptions
    /// An interrupt vector fired that should be impossible in long mode (reserved).
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
    /// No usable memory-map entry was large enough to hold the PMM bitmap.
    NoValidMemMapEntry = 0x0300,
    /// `pmm::alloc` or `pmm::free` was called before `pmm::init`.
    PmmNotInitialized = 0x0301,
    /// `pmm::free` was called on a frame that was already free.
    DoubleFree = 0x0302,
    /// `pmm::free` was called on a frame that must not be freed (frame 0, bitmap range, or out-of-range).
    IllegalFree = 0x0303,
    /// The PMM could not allocate a frame because all frames are in use.
    OutOfMemory = 0x0304,
    // Memory: Paging
    /// `vmm::unmap_page` was called on a virtual address with no present mapping.
    InvalidPageOperation = 0x320,

    // Display
    /// The PSF2 font file embedded in the binary has the wrong magic number.
    InvalidPsf2MagicNumber = 0x1000,
}

impl PanicCode {
    /// Return the variant name as a static string, suitable for printing in a panic message.
    pub fn as_str(&self) -> &'static str {
        match self {
            // General
            PanicCode::Unknown => "Unknown",
            PanicCode::ManuallyTriggeredPanic => "ManuallyTriggeredPanic",
            PanicCode::InitFailure => "InitFailure",

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
