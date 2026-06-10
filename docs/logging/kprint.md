# kprint! — Kernel Print Macro

**Source:** `src/kernel/src/logging/kprint.rs`

---

## Overview

`kprint!` is the kernel's logging macro. It accepts the same format string syntax as `std::print!` and dispatches output to both COM1 serial and the basic framebuffer simultaneously.

---

## How It Works

Three layers:

1. **`kprint!` macro** — calls `core::fmt::write` with a `KernelWriter` instance and the result of `format_args!(...)`. `format_args!` constructs a `core::fmt::Arguments` value at compile time without allocating; `core::fmt::write` drives it by calling `write_str` repeatedly.

2. **`KernelWriter`** — a zero-size struct that implements `core::fmt::Write`. Its `write_str` method calls the private `kprint(string)` function.

3. **`kprint(string)`** — calls `super::serial::write_to_serial(string)` and `crate::screen::basic::text::print_to_basic_fb(string)` in sequence.

The `format_args!` / `core::fmt::Write` pattern is the standard no-alloc formatting approach in `no_std` environments — it formats directly into a writer without an intermediate heap-allocated `String`.

---

## Usage

```rust
kprint!("hello\n");
kprint!("value = {:#x}\n", some_u64);
kprint!("{some_struct:#?}\n");   // requires Debug impl
```

The macro is exported with `#[macro_export]`, so it's accessible as `crate::kprint!` from anywhere in the kernel without an explicit `use`.

---

## Output Destinations

Both outputs are unconditional — there is no way to send to serial only or framebuffer only via `kprint!`. For serial-only output (e.g. in `#[panic_handler]` where a second panic must be avoided), call `write_to_serial` directly.

If the framebuffer isn't initialized yet (i.e. `get_framebuffer()` returns `None`), `print_to_basic_fb` returns early and silently drops the output. Serial output is always available after `init_serial()`.
