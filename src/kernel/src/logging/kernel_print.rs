//! kernel_print.rs
//! Logging Macros
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

pub fn kprint(string: &str) {
    super::serial::write_string_to_com1(string);
    super::vga::print(string);
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
        use core::fmt::Write;
        let _ = core::fmt::write(
            &mut $crate::logging::kernel_print::KernelWriter,
            format_args!($($arg)*)
        );
    });
}