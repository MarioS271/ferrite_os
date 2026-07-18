# Basic Framebuffer

**Source:** `src/kernel/src/screen/basic/framebuffer.rs`

---

## Overview

A thin wrapper around the linear framebuffer provided by Limine. Used for early-boot text output and panic display. "Basic" because it has no window system, compositing, or hardware acceleration — it writes pixels directly.

---

## State

```
static BASIC_FRAMEBUFFER: Once<BasicFramebufferData>

pub struct BasicFramebufferData {
    pub fb_pointer:   *mut u32,   // base address of the framebuffer in virtual memory
    pub pixel_stride: u32,        // pixels per row (pitch / 4)
    pub width:        u64,
    pub height:       u64,
}
```

`fb_pointer` is `*mut u32` — each pixel is a 32-bit value in `0x00RRGGBB` format (the high byte is unused/ignored). `pixel_stride` is pitch divided by 4 (bytes-per-pixel), giving the number of 32-bit steps to move one row down. Limine reports pitch in bytes; the division converts it to pixels.

`Send` and `Sync` are manually implemented because `*mut u32` is not automatically `Send`/`Sync`. This is safe: the `Once` guarantees that the pointer is written exactly once before any reads, and the kernel is single-threaded at the time the framebuffer is initialized and used.

---

## `init_framebuffer(limine_fb)`

Takes a `&Framebuffer` from Limine's `FramebufferRequest` response and stores its fields in `BASIC_FRAMEBUFFER` via `call_once`. Must be called before any drawing functions.

---

## `get_framebuffer() -> Option<&'static BasicFramebufferData>`

Returns `Some` if the framebuffer has been initialized, `None` otherwise. Callers that need to draw check this first. `kernel_panic` uses the `Option` to decide whether to attempt framebuffer rendering.

---

## `fb.clear(&self)`

Fills the entire framebuffer with `0u32` (black) by iterating over every pixel and writing with `write_volatile`. `write_volatile` is used to prevent the compiler from optimizing away the writes, since the memory is mapped I/O-like and the compiler has no way to know the writes are observable.

Pixel address formula: `fb_pointer + y * pixel_stride + x`.
