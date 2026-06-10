# GDT — Global Descriptor Table

**Source:** `src/kernel/src/arch/x86_64/gdt.rs`

---

## Overview

The GDT defines the memory segments the CPU uses to validate code and data access. On x86_64 in 64-bit mode, most segmentation is bypassed, but the GDT is still required to:
- Load a valid code segment selector into CS (the CPU checks this on every privilege transition).
- Register the TSS descriptor so `ltr` (load task register) can point the CPU at the TSS.

---

## State

```
static GLOBAL_DESCRIPTOR_TABLE: Once<GdtData>

struct GdtData {
    global_descriptor_table: GlobalDescriptorTable,
    code_selector:           SegmentSelector,
    tss_selector:            SegmentSelector,
}
```

`GdtData` stores the table and the two selectors that need to be loaded after it. The data segment selector is appended to the GDT but its selector value isn't stored because DS/ES/FS/GS don't need to be reloaded in 64-bit mode.

---

## `init()`

Steps:
1. Creates a `GlobalDescriptorTable` and appends three descriptors in order: `kernel_code_segment()`, `kernel_data_segment()`, `tss_segment(...)`. Each `append` call returns a `SegmentSelector`; only the code and TSS selectors are saved.
2. Stores everything in `GLOBAL_DESCRIPTOR_TABLE` via `call_once`.
3. Calls `.load()` on the table — writes the GDT base+limit to the GDTR register.
4. Calls `CS::set_reg(code_selector)` — reloads the CS register with the new kernel code segment. Required because the GDT pointer changed; the CPU's cached CS descriptor is now stale.
5. Calls `load_tss(tss_selector)` — loads the TSS selector into the task register (TR).

Steps 4 and 5 are `unsafe`: `CS::set_reg` is unsafe because an invalid selector would immediately fault; `load_tss` is unsafe for the same reason. Both are safe here because the selectors were just appended to the valid GDT.

---

## Descriptor Order

The order of `append` calls matters — segment selectors are offsets into the GDT, so the CPU interprets them by position. The standard kernel layout is: null (implicit, always index 0), kernel code, kernel data, TSS. Changing the order without updating the selectors would load the wrong descriptor into CS or TR.

---

## Dependency

Requires `tss::TASK_STATE_SEGMENT` to be initialized first — `Descriptor::tss_segment` takes a reference to the TSS.
