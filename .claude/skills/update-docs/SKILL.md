---
name: update-docs
description: Write or update rustdoc comments in FerriteOS. Use when asked to document, add rustdoc, update docs, or fix comments.
---

Write or update rustdoc for FerriteOS source files. Read the file first, then edit in place. Do not commit changes — present what was changed and wait for user approval before running any `git commit`.

## The rule that overrides everything

**A doc gives a baseline: what the thing is and what purpose it serves. Nothing more.**

It answers "what is this and why would I use it?" — never "how does it work?", "what does it call?", or "what happens step by step?". Exact workings, code paths, call sequences, initialization order, and internal state changes do not belong in doc comments. That information lives in the code, and the code is right there.

One sentence is the target. Reach for a second sentence only to state a constraint the reader cannot see from the name and type (a unit, an invariant, a safety condition). If a sentence would not change how someone *uses* the item, cut it.

When judging existing docs, delete over-explanation as readily as you add missing docs. A three-line doc that walks through the mechanism is worse than a one-line doc that states the purpose.

## Before / after

These show the transformation to apply. The "after" is the goal.

**Struct — remove the mechanism, keep the purpose:**

```
// BAD — describes init path and caller protocol (workings)
/// Global early-boot state, accessible from anywhere in the kernel.
///
/// Fields are initialized in `kmain` via `call_once` and then read-only. Callers
/// should check `is_completed()` before calling `get()` — for example, the panic
/// handler does this to avoid crashing while printing a crash message.

// GOOD
/// Global early-boot state, readable from anywhere in the kernel once initialized.
```

**Module — one line of purpose, not an essay on the algorithm:**

```
// BAD — explains the four-level walk, the copy of entries 256–511, the CR3 write
//! Virtual Memory Manager (VMM) for x86_64.
//!
//! Owns the kernel's PML4 page table... [three paragraphs of mechanism]

// GOOD
//! Virtual Memory Manager (VMM) for x86_64: owns the kernel page table and maps
//! and unmaps pages at 4 KiB granularity.
```

**Struct — state what it is for, not how the lock works:**

```
// BAD
/// A spinlock that saves and restores the CPU's Interrupt Flag (RFLAGS.IF).
/// When lock() is called, the current IF state is captured and interrupts are
/// disabled. When the guard is dropped... [mechanism]

// GOOD
/// Spinlock for data shared between normal kernel code and interrupt handlers;
/// disables interrupts while held to avoid self-deadlock.
```

## File header

Every `.rs` file starts with the SPDX line as a standalone `//` comment on line 1 — never inside a `//!` block:

```rust
// SPDX-License-Identifier: GPL-3.0-only
```

Module docs follow it:

```rust
// SPDX-License-Identifier: GPL-3.0-only
//! One line: what this module is responsible for.
//!
//! Authors: MarioS271
```

## Style rules

**Module-level (`//!`)** — one line of purpose. A second line only for a caller-facing invariant. Never list the types inside, the init sequence, or what the module calls.

**Struct / trait-level (`///`)** — one line of purpose. A second line only for a constraint the fields don't reveal (e.g. "written once at boot, then read-only"). Document a field only when its name and type leave the meaning or unit unclear.

**Function-level (`///`)** — one line, active verb: what it does and returns ("Allocate one free physical frame and return its address."). Add `# Panics` naming the condition when it can panic. Add `# Safety` stating the caller's invariant when it is `unsafe`. Add `# Examples` (` ```ignore ``` `) only for public API where usage is genuinely non-obvious.

## What to document

- **File headers** (`//!`) — always
- **Structs and traits** (`///`) — always; fields only when the name alone is not enough
- **Functions / methods** (`///`) — always

Everything else — constants, statics, type aliases, enums, enum variants, `impl` blocks, `mod` declarations — gets a doc **only if the name and value together leave something genuinely unclear**. When in doubt, leave it undocumented. Do not add rustdoc to individual panic codes (`panic.rs`).
