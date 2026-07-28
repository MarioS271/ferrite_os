---
name: update-docs
description: Write or update rustdoc comments in FerriteOS. Use when asked to document, add rustdoc, update docs, or fix comments.
---

Write or update rustdoc for FerriteOS source files. Read the file first, then edit in place. Do not commit changes — present what was changed and wait for user approval before running any `git commit`.

Keep docs short and simple, but informative. Every sentence should tell the reader something they couldn't figure out from the name alone.

## File header

Every `.rs` file starts with the SPDX line as a standalone `//` comment on line 1 — never inside a `//!` block:

```rust
// SPDX-License-Identifier: GPL-3.0-only
```

## Style rules

**Module-level (`//!`) — all files**

```rust
// SPDX-License-Identifier: GPL-3.0-only
//! One-line description of what the code in here does.
//!
//! Further, more precise docs go here (if necessary).
//!
//! Authors: MarioS271
```

- State what the module owns and what it does in the kernel
- Name exact types involved (`PageTable`, `IrqMutex<T>`, etc.)
- Describe the initialization sequence and any non-obvious invariants
- End with `Authors: MarioS271`
- Two to five sentences is the target length; routing `mod.rs` files naturally end up shorter since they have less to describe

**Struct-level (`///`)**
- One-line summary describing purpose, not just restating the name
- Describe non-obvious invariants (e.g. "written once during init, then read-only")
- Document each field with one line covering what it holds and any constraint

**Function-level (`///`)**
- First line: active verb, short ("Allocate one free physical frame and return its address.")
- Body: describe the sequence of operations with exact variable/type names, not abstract prose
- Add `# Panics` when the function can panic, naming the condition
- Add `# Safety` when the function is `unsafe`, stating the invariant the caller must uphold
- Add `# Examples` with ` ```ignore ``` ` only for public API that benefits from a usage example (rare in kernel code)

**Field-level (`///`)**
- One line. State what the value represents and any constraint or unit.

## What not to do

- Do not restate what the name already says ("cursor_x: the x position of the cursor")
- Do not describe general concepts the reader already knows ("a page table maps virtual to physical")
- Do not write multi-line or multi-paragraph docs for simple items
- Do not add comments explaining WHAT code does — only WHY when non-obvious
- Do not describe what the reader can see by scanning the code — if the doc just restates the function signatures, bullet-lists the branches, or walks through the control flow, delete it and write one sentence about the non-obvious part instead
- Do not explain the mechanism when the outcome is enough: "prevents infinite panic re-entry" beats "set to true the first time X fires; prevents Y from looping back through Z"

## Reference examples

Module doc — `src/kernel/src/mem/x86_64/vmm.rs:1`
Struct + fields — `src/kernel/src/types/irq_mutex.rs:22`
Function with `# Panics` — `src/kernel/src/mem/pmm.rs:52`
Function with `# Safety` — `src/kernel/src/types/irq_mutex.rs:62`
Macro doc — `src/kernel/src/logging/kprint.rs` (`kprint!`)
