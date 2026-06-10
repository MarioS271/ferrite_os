# Panic Handler

**Source:** `src/kernel/src/panic.rs`

---

## Overview

Two separate panic entry points exist:

- `kernel_panic()` — the kernel's own structured panic. Called explicitly by kernel code when something goes wrong (bad memory state, unrecoverable exception, etc.).
- `#[panic_handler] fn panic()` — Rust's required panic handler, fired when the Rust runtime itself panics (e.g. a failed `unwrap()`, bounds check, or integer overflow in debug mode). Serial-only.

---

## `kernel_panic(panic_code, panic_message, print_debug_text) -> !`

Takes a `PanicCode` (`types/panic_codes.rs` — `#[repr(u16)]` enum with `as_str()`), a message string, and a boolean controlling whether to render to the framebuffer.

Steps:
1. Calls `arch::instructions::disable_interrupts()` — no further interrupt delivery after this point.
2. If `print_debug_text` is `true` and a framebuffer is available (`get_framebuffer()` returns `Some`): clears the framebuffer, then renders "Kernel Panic!", the `PanicCode` name, and the message string in red via `draw_string`.
3. Always writes the same information to COM1 serial via `write_to_serial`.
4. Loops forever calling `arch::instructions::halt_cpu()` (`hlt` instruction). Halts but remains in the loop in case an NMI wakes the CPU.

`print_debug_text: false` is used in early boot paths where the framebuffer may not be initialized yet (e.g. `PanicCode::InvalidPsf2MagicNumber` is triggered during font parsing, before the framebuffer is confirmed stable).

---

## `#[panic_handler] fn panic(panic_info)`

Rust's ABI-required handler. Fires on any unhandled Rust runtime panic.

Steps:
1. `disable_interrupts()`
2. Writes "FATAL: rustlang panic handler fired!" and `panic_info.message()` to serial. If the message is missing, falls back to `"(No panic info message given)"`.
3. Loops on `halt_cpu()`.

No framebuffer output — this handler may fire before the framebuffer is initialized, and keeping it serial-only avoids a second panic inside the handler.

---

## Color Convention

`kernel_panic` renders "Kernel Panic!" in `0x00FF0000` (red). All subsequent text uses `None` (defaults to white). This is hardcoded — no theme system.
