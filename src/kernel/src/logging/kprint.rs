//! logging/kprint.rs
//! Logging Macros
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

fn kprint(string: &str) {
    super::serial::write_to_serial(string);
    crate::screen::basic::text::print_to_basic_fb(string);
}

pub struct KernelWriter;
impl core::fmt::Write for KernelWriter {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        kprint(string);
        Ok(())
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ({
        let _ = core::fmt::write(
            &mut $crate::logging::kprint::KernelWriter,
            format_args!($($arg)*)
        );
    });
}
