# IDT — Interrupt Descriptor Table

**Source:** `src/kernel/src/arch/x86_64/idt.rs`

---

## Overview

The IDT maps CPU exception vectors and hardware interrupt vectors to handler functions. When the CPU takes an exception (or interrupt), it indexes into the IDT with the vector number and jumps to the registered handler.

---

## State

```
static INTERRUPT_DESCRIPTOR_TABLE: Once<InterruptDescriptorTable>
```

The IDT is stored in a `Once` so it lives for the lifetime of the kernel. The CPU holds a pointer to it (in the IDTR register) and dereferences it on every exception — it must never be freed or moved.

---

## `init()`

Steps:
1. Creates an `InterruptDescriptorTable` and registers three handlers:
   - `double_fault` — uses `.set_stack_index(0)` to switch to IST[0] (the dedicated double fault stack from `tss.rs`) on delivery. This is `unsafe` because an invalid stack index would cause a triple fault.
   - `general_protection_fault` — standard handler, no IST switch.
   - `page_fault` — standard handler, no IST switch.
2. Stores the IDT in `INTERRUPT_DESCRIPTOR_TABLE` via `call_once`.
3. Calls `.load()` on the stored IDT — writes the IDT base+limit to the IDTR register.

The IDT must stay at a fixed address while loaded. Storing it in a `Once<...>` static satisfies this — the address never changes after `call_once`.

---

## Registered Handlers

| Vector | Exception               | Handler file                          | IST |
|--------|-------------------------|---------------------------------------|-----|
| #8     | Double fault            | `exceptions/double_fault.rs`          | 0   |
| #13    | General protection fault| `exceptions/general_protection_fault.rs` | — |
| #14    | Page fault              | `exceptions/page_fault.rs`            | —   |

All other vectors remain unregistered. An unregistered exception will triple fault for now. Hardware interrupts are also unregistered — the PIC/APIC is not yet initialized.

---

## Dependency

Requires `gdt::init()` to have run first — the GDT must be loaded and the TSS registered before the IDT can use IST entries.
