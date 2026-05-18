//! serial.rs
//! Serial Logging on COM1
//!
//! Authors: MarioS271
//! Licensed via the AGPLv3 license

use x86_64::instructions::port::Port;

static COM1_PORT_BASE_ADDRESS: u16 = 0x03F8;

pub fn init_com1() {
    let at_offset = |offset: u16| -> Port<u8> {
        Port::<u8>::new(COM1_PORT_BASE_ADDRESS + offset)
    };

    // Safe because:
    // 1) static mut
    //    No threading exists at the time of this being executed, as this function gets called
    //    as one of the first things in kmain
    // 2) at_offset().write()
    //    Safe because only writing to common UART registers (follows the standard 16550 spec,
    //    therefore will not corrupt unrelated things)
    unsafe {
        static mut WAS_ALREADY_CALLED: bool = false;

        if WAS_ALREADY_CALLED {
            return;
        }

        WAS_ALREADY_CALLED = true;

        at_offset(1).write(0x00);     // Disable Interrupts.
        at_offset(3).write(0x80);     // Enable DLAB (divisor latch access bit)
        at_offset(0).write(0x01);     // Baud Rate Divisor (low byte)
        at_offset(1).write(0x00);     // Baud Rate Divisor (high byte)
        at_offset(3).write(0x03);     // Disable DLAB and configure line
        at_offset(2).write(0xC7);     // Enable and clear FIFO
        at_offset(4).write(0x0B);     // Set DTR, RTS, OUT2
    }
}

pub fn write_string_to_com1(string: &str) {
    let mut com1_port: Port<u8> = Port::new(COM1_PORT_BASE_ADDRESS);

    for byte in string.bytes() {
        // This calls write_byte which was already declared to be safe because
        // it adheres to 16550 spec
        unsafe {
            com1_port.write(byte);
        }
    }
}