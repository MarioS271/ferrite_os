# Font — PSF2 Rendering

**Source:** `src/kernel/src/screen/basic/font.rs`

---

## Overview

Parses and renders a PSF2 (PC Screen Font version 2) bitmap font. The font is embedded in the kernel binary at compile time via `include_bytes!`. Character rendering produces individual glyphs onto the framebuffer at a given pixel coordinate.

Font in use: `ter-powerline-v16n.psf` (Terminus 16px, Powerline variant). Embedded as `static TERMINUS_POWERLINE_16: &[u8]`.

---

## PSF2 Format

A PSF2 file starts with a fixed header, followed by a flat array of glyph bitmaps. The header fields used here:

| Field            | Byte offset | Meaning |
|------------------|-------------|---------|
| magic            | 0           | Must equal `0x864ab572` (little-endian u32) |
| header_size      | 8           | Byte offset where glyph data starts |
| glyph_count      | 16          | Number of glyphs in the file |
| bytes_per_glyph  | 20          | Byte size of one glyph's bitmap |
| height           | 24          | Glyph height in pixels |
| width            | 28          | Glyph width in pixels |

The glyph for character `c` starts at `header_size + (c as usize) * bytes_per_glyph`. Each row of the glyph is stored as a packed sequence of bits, with each row padded to a whole number of bytes: `row_stride = (width + 7) / 8`. Bit 7 of the first byte in a row is the leftmost pixel.

---

## State

```
pub static TERMINUS_POWERLINE_16_HEADER: Once<Psf2Header>
```

`Psf2Header` stores the five header fields listed above. Parsed once at init; all rendering functions read from it.

---

## `init_font_header()`

Calls `parse_psf2_header(TERMINUS_POWERLINE_16)`. Checks the magic number and panics with `PanicCode::InvalidPsf2MagicNumber` if it doesn't match — the embedded bytes are corrupted or not a PSF2 file. Stores the result in `TERMINUS_POWERLINE_16_HEADER`.

---

## `draw_char(fb, c, x, y, font_color) -> usize`

Renders one character onto `fb` at pixel position `(x, y)`.

Steps:
1. Gets the header from `TERMINUS_POWERLINE_16_HEADER`; returns `x` unchanged if not initialized.
2. Gets the glyph slice for `c` from `parse_char`; returns `x` if the character is out of range.
3. Bounds-checks that the glyph fits inside the framebuffer; returns `x` if it doesn't.
4. For each row and column of the glyph: extracts the bit at that position, maps it to `font_color` (or white if `None`) for a set bit or `BACKGROUND_COLOR` (`0x00000000`, black) for a clear bit, and writes the color value to the framebuffer.
5. Returns `x + header.width` — the x coordinate immediately after the drawn character.

The caller uses the return value to chain characters horizontally without tracking width itself.

---

## Colors

- Background: `0x00000000` (black) — hardcoded, not configurable per-call.
- Foreground: caller-supplied `Option<u32>`, falls back to `0x00FFFFFF` (white) if `None`.
- Format: `0x00RRGGBB`.
