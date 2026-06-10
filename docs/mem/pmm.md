# PMM — Physical Memory Manager

---

## Design

Bitmap allocator. One bit per 4 KiB physical frame.

```
bit = 0  →  frame free
bit = 1  →  frame used
```

The bitmap is placed inside physical memory itself — in the first usable region large enough to hold it. The frames that the bitmap occupies are permanently marked used so the allocator never hands them out.

---

## Initialization

`init(entries, hhdm_offset)` takes Limine's memory map slice and the HHDM offset.

Steps:
1. Scan usable entries to find the highest physical address. Divide by 4096 to get `total_frames`.
2. Compute `bitmap_bytes = total_frames.div_ceil(8)`.
3. Find the first usable entry whose `length >= bitmap_bytes`; that entry's base becomes `bitmap_physical_base_addr`.
4. Zero-fill the entire bitmap (marks every frame used: `0xFF`).
5. Record `bitmap_start_frame` and `bitmap_end_frame` (the range of frames occupied by the bitmap itself).
6. Walk usable entries again; for each frame in each entry, clear its bit (mark free) — skipping frame 0 and any frame inside the bitmap range.
7. Store everything in `Once<PmmData>`.

---

## Invariants

- Frame 0 is never freed during init and `free()` panics if you try to free it.
- All frames default to used; only explicitly usable regions are freed during `init()`.
- The frames containing the bitmap are never freed.
- Limine-reclaimable memory is **not** freed during `init()` — only `MEMMAP_USABLE` entries are processed.

---

## Public API

```rust
pub fn init(entries: &[&memmap::Entry], hhdm_offset: u64)
pub fn alloc() -> Option<PhysAddr>   // returns 4 KiB-aligned physical address, or None if OOM
pub fn free(addr: PhysAddr)
```

`stats()` does **not** exist yet — the design doc mentioned it but it is not implemented.

### `alloc()`

Scans the bitmap byte-by-byte. Skips bytes that are `0xFF` (all frames in that byte are used). On a non-full byte, uses `u8::trailing_ones(byte)` to find the first free bit, marks it used, and returns the corresponding `PhysAddr`. Returns `None` if the entire bitmap is full.

### `free(addr)`

Panics (via `kernel_panic`) on:
- Frame 0 (`PanicCode::IllegalFree`)
- Any frame inside the bitmap range (`PanicCode::IllegalFree`)
- Any frame beyond `total_frames` (`PanicCode::IllegalFree`)
- A frame whose bit is already 0 (double-free) (`PanicCode::DoubleFree`)

Otherwise clears the bit.

---

## Internal State

```rust
struct PmmData {
    bitmap_ptr:         *mut u8,
    total_frames:       u64,
    bitmap_start_frame: u64,   // first frame the bitmap occupies
    bitmap_end_frame:   u64,   // last frame the bitmap occupies (inclusive)
}
```

Stored in `Once<PmmData>`. Both `alloc()` and `free()` call `kernel_panic` with `PanicCode::PmmNotInitialized` if called before `init()`.

---

## Frame Size

`pub static FRAME_SIZE: u64 = 4096` — exported so VMM and other callers can use it without a magic number.
