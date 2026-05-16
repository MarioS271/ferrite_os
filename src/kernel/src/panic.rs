//! panic.rs
//! Panic Handler
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("cli; hlt", options(nostack, nomem))
        }
    }
}