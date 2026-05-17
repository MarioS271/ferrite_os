//! panic.rs
//! Panic Handler
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use crate::panic::font;
use crate::panic::framebuffer;
use core::arch::asm;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    // Necessary to disable any interrupts to prevent the panic sequence from being interrupted
    unsafe {
        asm!("cli", options(nostack, nomem))
    }

    // TODO: render actual text on panic
    if let Some(panic_fb) = panic_framebuffer::get_framebuffer() {
        let pixels: usize = panic_fb.pixel_stride as usize * panic_fb.height as usize;

        for i in 0..pixels {
            // add() increments the target address and write_volatile() writes to that address.
            // As long as limine gives us valid panic data, this stays safe as it gets
            // limited by pixels as the max iterator
            unsafe {
                panic_fb.fb_pointer.add(i).write_volatile(0x00FF0000)
            };
        }

        font::draw_char('a');
    }

    loop {
        // Halt the cpu for saving resources as system is gone anyway
        unsafe {
            asm!("hlt", options(nostack, nomem))
        }
    }
}