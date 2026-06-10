# Text — Framebuffer String Drawing

**Source:** `src/kernel/src/screen/basic/text.rs`

---

## Overview

Two modes of writing text to the framebuffer:

- `draw_string` — stateless, caller-owned cursor. Used by `kernel_panic` which needs full control over where text appears.
- `print_to_basic_fb` — stateful, global cursor with automatic scrolling. Used by `kprint!` for continuous kernel log output.

---

## `draw_string(fb, string, x, y, font_color)`

Iterates over `string.chars()`. On `\n`, resets `x` to 0 and increments `y` by `font_header.height`. For any other character, calls `draw_char` and uses its return value as the new `x`.

Takes `x` and `y` as `&mut usize` — the caller's variables are updated in place, so chained calls naturally continue from where the last one left off.

`draw_string_no_mut` is a wrapper that takes `x`/`y` by value and returns the updated pair as a tuple, for callers that don't want to deal with mutable references.

Returns early (silently) if the font header hasn't been initialized yet.

---

## `print_to_basic_fb(string)`

The entry point used by `kprint!`. Manages a global `Mutex<VgaPrintState>` holding the current cursor position.

Steps:
1. Locks `VGA_PRINT_STATE`.
2. Calls `draw_string` with the locked cursor position.
3. Writes the updated cursor back to the state.
4. If the cursor's `y` has moved past the last visible row (`fb.height - font_header.height`), clamps `y` to that last row and scrolls.

### Scrolling

Scrolling is a single `core::ptr::copy` followed by a clear of the bottom row:

1. `copy(src, dst, count)` — copies from `fb_pointer + font_height * pixel_stride` to `fb_pointer`, moving every pixel up by one glyph row. `src` and `dst` overlap for all but the first row, but `core::ptr::copy` (equivalent to `memmove`) handles overlapping regions correctly.
2. Clears the last row by writing `0u32` to each pixel in the range `(height - font_height) * pixel_stride .. height * pixel_stride`.

This is a full pixel blit every time the cursor overflows — fine for a kernel debug console, not suitable for a high-frequency display path.

---

## Mutex vs. Lock-Free

`VGA_PRINT_STATE` uses `spin::Mutex`. The basic framebuffer itself (`BASIC_FRAMEBUFFER`) is lock-free (`Once`) because it's needed from the panic handler, which must not deadlock on a lock held by the thread that panicked. The text cursor state is different — it's mutable on every write, so a mutex is appropriate here. The panic handler bypasses `print_to_basic_fb` entirely and calls `draw_string` directly with its own coordinates.
