//! logging/kernel_print.rs
//! Logging Macros
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

pub fn kprint(string: &str) {
    super::serial::write_string_to_com1(string);
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
            &mut $crate::logging::kernel_print::KernelWriter,
            format_args!($($arg)*)
        );
    });
}