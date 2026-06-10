# TSS — Task State Segment

**Source:** `src/kernel/src/arch/x86_64/tss.rs`

---

## Overview

The TSS is required on x86_64 to define the Interrupt Stack Table (IST) — a set of known-good stack pointers the CPU switches to when delivering certain exceptions. Without it, a stack overflow or corrupted RSP would make the double fault handler itself fault, causing a triple fault and reset.

Only IST entry 0 is used, and only for the double fault handler.

---

## State

```
pub static TASK_STATE_SEGMENT: Once<TaskStateSegment>
static mut DOUBLE_FAULT_STACK: DoubleFaultStack   // 4 KiB, align(16)
```

`DoubleFaultStack` is a newtype around `[u8; 4096]` with `#[repr(align(16))]`. The 16-byte alignment is required because the CPU pushes the interrupt frame to this stack and the x86_64 ABI requires RSP to be 16-byte aligned before a call. Without it, any SSE instruction in the double fault handler would crash.

`TASK_STATE_SEGMENT` is `pub` so `gdt.rs` can reference it when building the GDT's TSS descriptor.

---

## `init()`

Steps:
1. Computes the top-of-stack address for `DOUBLE_FAULT_STACK` — the pointer to the last element plus one (stacks grow downward, so the "top" is the highest address).
2. Creates a `TaskStateSegment` via `TaskStateSegment::new()`, sets `tss.interrupt_stack_table[0]` to that address.
3. Stores the TSS in `TASK_STATE_SEGMENT` via `call_once`.

The `unsafe` block is justified by single-threaded execution at this point — no other code can race on `DOUBLE_FAULT_STACK` during early boot.

---

## Relationship to GDT and IDT

- `gdt::init()` reads `TASK_STATE_SEGMENT` to create a `Descriptor::tss_segment(...)` entry and appends it to the GDT, then calls `load_tss` with the resulting selector.
- `idt::init()` calls `.set_stack_index(0)` on the double fault entry, pointing it at IST[0].

TSS must be initialized before GDT, which must be initialized before IDT. `arch::init()` enforces this order.
