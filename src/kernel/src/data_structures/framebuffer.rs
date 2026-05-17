//! framebuffer.rs
//! Global Framebuffer Struct
//!
//! The panic panic is the same as the basic panic, just accessed differently to
//! prevent deadlocks when panicking while the basic panic mutex is locked
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

pub struct FramebufferData {
    pub fb_pointer: *mut u32,
    pub pixel_stride: u32,
    pub width: u64,
    pub height: u64,
    pub x: usize,
    pub y: usize,
}

// TODO: update unsafe comment
// This is safe because PanicFramebufferData is wrapped in a Once().
// The data will only be written BEFORE parallelism exists.
// Afterward, the data will become read-only, making Send and Sync thread-safe.
unsafe impl Send for FramebufferData {}
unsafe impl Sync for FramebufferData {}