# Serial — COM1 Serial Output

**Source:** `src/kernel/src/logging/x86_64/serial.rs`

---

## Overview

Writes text to the COM1 serial port using 16550 UART port I/O. The serial port is the earliest output available — it can be written to before the framebuffer is initialized, and QEMU pipes it to the host terminal by default.

COM1 base address: `0x03F8`. All register access is via x86 port I/O (`Port<u8>` from the `x86_64` crate), not MMIO.

---

## `init_serial()`

Configures the 16550 UART by writing to its registers in a specific sequence. Each register is accessed as `base_address + offset`:

| Offset | Register (when DLAB=0) | Write sent    | Effect |
|--------|------------------------|---------------|--------|
| +1     | Interrupt Enable       | `0x00`        | Disable all UART interrupts |
| +3     | Line Control           | `0x80`        | Set DLAB=1 to access divisor registers |
| +0     | Divisor Latch (low)    | `0x01`        | Set baud rate divisor low byte |
| +1     | Divisor Latch (high)   | `0x00`        | Set baud rate divisor high byte |
| +3     | Line Control           | `0x03`        | Clear DLAB; set 8 data bits, no parity, 1 stop bit |
| +2     | FIFO Control           | `0xC7`        | Enable FIFO, clear TX/RX FIFOs, set 14-byte trigger level |
| +4     | Modem Control          | `0x0B`        | Set DTR, RTS, OUT2 |

Divisor 1 at the default 115200 baud clock gives 115200 baud.

A `static mut WAS_ALREADY_CALLED: bool` guard prevents double initialization. Safe because `init_serial` is called once in single-threaded early boot before any interrupts are enabled.

---

## `write_to_serial(string)`

Iterates over the bytes of the string and writes each byte to port `0x03F8` (the TX holding register). Does not wait for the transmit buffer to empty — assumes FIFO is enabled and won't overflow at kernel logging rates. No null terminator or framing added.

Called directly by `kprint`'s inner function and by the `#[panic_handler]` (which bypasses `kprint` to avoid a second panic).

---

## No `read_from_serial`

There is no read path yet. Reading from serial will be needed once a serial console or interactive input is required.
