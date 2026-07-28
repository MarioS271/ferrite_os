---
name: update-docs
description: Write or update rustdoc comments in FerriteOS. Use when asked to document, add rustdoc, update docs, or fix comments.
---

Write or update rustdoc for FerriteOS source files. Read the file first, then edit in place.

## Style rules

**Module-level (`//!`) — `mod.rs` files**

`mod.rs` files are routing modules: they only declare submodules and re-export their contents. Their doc block is intentionally minimal — two `//!` blocks separated by a blank line:

```
//! path/to/mod.rs
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only

//! One-line description of what this module group contains.
```

The first block is just the file path and license header. The second block is a single sentence naming the subsystem and listing the key submodules or responsibilities. No types, no initialization detail — those belong in the implementation files. See `src/kernel/src/arch/mod.rs` and `src/kernel/src/logging/mod.rs` as examples.

**Module-level (`//!`) — implementation files (e.g. `vmm.rs`, `kprint.rs`)**

Example:

```
//! path/to/mod.rs
//! One-line description of what the code in here does.
//!
//! Further, more precise docs go here.
//!
//! Authors: MarioS271
//! SPDX-License-Identifier: GPL-3.0-only
```

Implementation files get a full module doc:
- State what the module owns and what it does in the kernel
- Name exact types involved (`PageTable`, `IrqMutex<T>`, etc.)
- Describe the initialization sequence and any non-obvious invariants
- End with `Authors: MarioS271` and `SPDX-License-Identifier: GPL-3.0-only`
- Two to five sentences is the target length; more is fine when the design needs explaining

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
- Do not add `// Safety:` inline comments unless the justification is non-obvious

## Reference examples

Module doc — `src/kernel/src/mem/x86_64/vmm.rs:1`
Struct + fields — `src/kernel/src/types/irq_mutex.rs:22`
Function with `# Panics` — `src/kernel/src/mem/pmm.rs:52`
Function with `# Safety` — `src/kernel/src/types/irq_mutex.rs:62`
Macro doc — `src/kernel/src/logging/kprint.rs` (`kprint!`)
