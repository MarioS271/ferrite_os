// SPDX-License-Identifier: GPL-3.0-only
//! VMM Helpers
//!
//! Authors: MarioS271

use crate::panic::kernel_panic;
use crate::types::panic_codes::PanicCode;

/// Panic when the VMM cannot allocate a frame from the PMM.
pub fn out_of_memory_panic() -> ! {
    kernel_panic(
        PanicCode::OutOfMemory,
        "VMM could not allocate a frame, out of memory",
    )
}

/// Panic when `unmap_page` hits a page-table level that is not present.
pub fn invalid_unmap_panic() -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        "Attempting to unmap a page without a PRESENT flag",
    );
}

/// Panic when `remap_page` hits a page-table level that is not present.
pub fn invalid_remap_panic() -> ! {
    kernel_panic(
        PanicCode::InvalidPageOperation,
        "Attempting to remap a page without a PRESENT flag",
    );
}