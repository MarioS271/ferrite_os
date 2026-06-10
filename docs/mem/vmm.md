# VMM — Virtual Memory Manager

**Status:** Partially implemented. (`src/kernel/src/mem/x86_64/vmm.rs`, `vmm_helpers.rs`)

`init`, `map_page`, and `unmap_page` are done and tested. Page fault handler integration, guard pages, CoW, and the higher-level memory management API (`mmap`, `brk`, etc.) are not yet written.

---

## Design

Sits above the PMM. Manages x86_64 four-level page tables (PML4 → PDP → PD → PT). The kernel owns its own PML4 table (not Limine's); Limine's kernel-half entries are copied in at init time.

State is stored in `Once<VmmData>` (`plm4_ptr`, `hhdm_offset`), accessible via `vmm::get()`.

---

## Initialization

`init(hhdm_offset)`:

1. Read Limine's current PML4 physical address from CR3.
2. Allocate a new PML4 frame from PMM; translate to virtual via `+ hhdm_offset`.
3. Zero the entire new PML4 table (`write_bytes(..., 0x00, 1)` — covers all 512 entries).
4. Copy entries 256–511 (kernel half) from Limine's PML4 into the new one. Entries 0–255 (user half) stay zeroed.
5. Write the new PML4's physical address into CR3, keeping the existing CR3 flags.
6. Store `plm4_ptr` and `hhdm_offset` in `Once<VmmData>`.

The zeroing in step 3 covers both halves. The user half ends up zeroed as a natural consequence, not as a separate targeted step.

---

## Public API

```rust
pub fn init(hhdm_offset: u64)
pub fn get() -> &'static VmmData
pub unsafe fn map_page(virt: VirtAddr, phys: PhysAddr, flags: PageTableFlags)
pub unsafe fn unmap_page(virt: VirtAddr)
```

Both `map_page` and `unmap_page` are `unsafe` because the caller controls what physical memory gets mapped and where.

### `map_page(virt, phys, flags)`

Walks P4 → P3 → P2, allocating intermediate page table frames as needed. Intermediate entries get `PRESENT | WRITABLE | (flags & USER_ACCESSIBLE)` — they inherit `USER_ACCESSIBLE` from the caller's flags and nothing else. Then sets the P1 entry to `phys` with `flags | PRESENT`.

If an intermediate entry already has `PRESENT` set, the existing frame is reused (no re-allocation).

### `unmap_page(virt)`

Walks P4 → P3 → P2 → P1. At each level, if the entry lacks `PRESENT`, calls `invalid_unmap_panic()`. At P1, if the entry lacks `PRESENT`, also panics. Then calls `entry.set_unused()` and flushes the TLB for `virt` via `x86_64::instructions::tlb::flush(virt)`.

Does **not** free the intermediate page table frames even if they become empty.

---

## Internal State

```rust
pub struct VmmData {
    pub plm4_ptr:    *mut PageTable,
    pub hhdm_offset: u64,
}
```

`Send` and `Sync` are manually implemented — access is safe because the `Once` guarantees single-writer initialization, and callers of `map_page`/`unmap_page` are responsible for their own synchronization.

---

## Helper Functions (`vmm_helpers.rs`)

- `alloc_zeroed_frame() -> PhysFrame` — allocates a frame from PMM, zeroes it via a `*mut PageTable` cast (so `write_bytes` with `count=1` covers the full 4096 bytes), returns a `PhysFrame`.
- `out_of_memory_panic() -> !` — called when PMM returns `None`; panics with `PanicCode::OutOfMemory`.
- `invalid_unmap_panic() -> !` — called when `unmap_page` walks into a non-PRESENT entry; panics with `PanicCode::InvalidPageOperation`.

---

## Virtual Address Allocation Notes

- P4 index 256 (`0xFFFF800000000000`–`0xFFFF807FFFFFFFFF`) — Limine's HHDM, mapped with huge pages. Do not allocate virtual addresses here.
- P4 index 257 onwards (`0xFFFF808000000000`+) — first safe kernel virtual region. The heap is placed at `0xFFFF_8080_0000_0000`.

---

## Planned

- Page fault handler: distinguish stack growth, CoW, lazy allocation, and actual faults
- Guard pages for kernel stacks
- Per-process address space management
- `mmap`, `munmap`, `mprotect`, `brk` (needed for Linux ABI)
- Copy-on-write fork support

### Linux compat notes (for future mmap/brk)

```rust
// mmap flags must match Linux uapi exactly
const MAP_SHARED:    u32 = 0x01;
const MAP_PRIVATE:   u32 = 0x02;
const MAP_FIXED:     u32 = 0x10;
const MAP_ANONYMOUS: u32 = 0x20;

// brk(0) returns current break
// brk(addr) returns current (unchanged) break on failure, not -1
```
