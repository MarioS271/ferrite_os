// SPDX-License-Identifier: GPL-3.0-only
//! ELF Loader
//!
//! Authors: MarioS271

pub(crate) mod defs;
pub(crate) mod loader;

#[repr(align(8))]
struct AlignedBytes<const N: usize> {
    bytes: [u8; N],
}
static USER_BINARY_ALIGNED: AlignedBytes<4656> = AlignedBytes {
    bytes: *include_bytes!("../../resources/user-binary"),
};
pub static USER_BINARY: &[u8] = &USER_BINARY_ALIGNED.bytes;
