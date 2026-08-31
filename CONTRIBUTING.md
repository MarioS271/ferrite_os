# Contributing to Ferrite

Thank you for wanting to contribute! :)
<br>
To keep the project safe and maintainable, please follow these guidelines.

## 1. Workflow

1. **Fork**: Create a fork of the repository in your own account.
2. **Branch**: Create a new branch for your changes (`git switch -c feature/my-feature`).
3. **Commits**: Write clear and descriptive commit messages. Use the standard commit message format (`feat(sched): `, `fix(mem-vmm):`, ...)
4. **Push**: Push the branch to your fork.
5. **Pull Request (PR)**: Open a PR against the `main` branch.

## 2. Pull Request Rules

- Briefly describe what you changed or added and why.
- Keep PRs small and focused on a single topic.
- **Never** commit `build.toml`, OVMF firmware files, or any other private configuration.
- Do not remove entries from or add unnecessary entries to ignore files (`.gitignore`, `.claudeignore`) unless absolutely necessary.

## 3. Code Style

- All kernel code is written in Rust; follow standard Rust naming conventions.
- For categorizing a flow you may use comments to create a distinction between code sections.
- Maintain the existing module and namespace structure (code goes under `src`, kernel stuff under `kernel`, …).
- Do not use comments to describe an obvious piece of code. Only use them to describe complex code.
- Write clean, 4-space indented and readable code.

### 3.1. Rustdoc
- Document all functions, methods, structs and statics with one line (or more, but only if the documented item requires) of rustdoc (should start with `///`)
- If the safety of a function/method or similar depends on what the caller does before and after it, that should be documented
  in the rustdoc in a `# Safety` section.
- If a function/method or similar has any code path which panics (via `kernel_panic`/`panic!` or similar that invokes the `#[panic_handler]`),
  this should be documented in the rustdoc in a `# Panics` section.

### 3.2. File Headers
File headers should look like the following:
```rust
// SPDX-License-Identifier: <your license>
//! <description of the contents of this file, example: "VMM Paging (x86_64): maps, unmaps, remaps and translates pages">
//!
//! Authors: <your name>
```
- Keep the file description short but informational
- Do not use offensive, illegal and/or NSFW names. The maintainer(s) reserve the right to reject any PR because of an unfitting name.
- The standard license for Ferrite is `GPL-3.0-only`. You may use one of your own, but please note that in your PR.
  Proprietary, closed-source licenses and similar are not allowed.
  The maintainer(s) reserve the right to reject any PR because of an unallowed/unfitting license.

### 3.3. Documenting Unsafe Code
- Keep `unsafe` blocks minimal and always document why they are necessary
- When documenting `unsafe` blocks, use a line comment (`//`) starting with `Safety: ` which should be located in the row above the `unsafe` block
- For `unsafe` blocks with inline assembly, do not document *why* the unsafe is there, rather document why the inline asm is needed.
- When documenting an unsafe block in a function or method, the unsafe block may also be documented in the functions/methods rustdoc, if it
  is logical to which unsafe block the rustdoc unsafe notice applies to (a good example is a simple one-line getter with an unsafe block
  which depends on the initialization state of the kernel)

## 4. Content

- This is a bare-metal OS kernel. Only contribute things that meaningfully advance the kernel itself (drivers, memory management, scheduling, etc.).
- Do not add new dependencies (`Cargo.toml`) without prior discussion. The `x86_64-unknown-none` target imposes strict constraints on what crates are usable.
- **Never** modify the linker script (`linker.ld`), the entry point, or the memory layout without prior approval, as these are load-bearing for the entire boot process.
- Custom features that only serve a personal use case belong in a fork, not the upstream repo.

## 5. AI-Generated Code

AI tools (GitHub Copilot, Claude, ChatGPT, Cursor, etc.) may be used to assist with development, but with strict requirements:

- **You are fully responsible.** The contributor who opens the PR owns every line, AI-assisted or not. If AI-generated code introduces a bug, security issue, or license problem, that is your responsibility. "The AI wrote it" is not acceptable.
- **You must understand what you submit.** Do not submit AI-generated code you cannot explain, justify, or defend line by line. If you cannot reason about it, do not include it.
- **Disclosure is mandatory.** Every commit that contains AI-assisted code **must** say so explicitly. Use the following in your commit message:

  ```
  Co-authored-by: AI <ai@tool>
  ```

  and include a note in the body, for example:

  ```
  This implementation was written with AI assistance (Claude / Copilot / etc.).
  All output was reviewed, tested, and is understood by the author.
  ```

- **PRs must disclose AI use prominently.** Add a dedicated section to your PR description:

  ```
  ## AI Assistance
  Portions of this PR were written or suggested by [tool name].
  The author has reviewed, tested, and takes full responsibility for all submitted code.
  ```

- **Review it like adversarial code.** AI code should be treated as untrusted. Audit for logic errors, unnecessary `unsafe`, incorrect memory semantics, or patterns that look plausible but are wrong for bare-metal targets.
- **No AI-generated `unsafe` blocks without explicit justification.** If AI produced an `unsafe` block, it must be fully replaced or accompanied by a human-written comment explaining exactly why it is safe. It must not merely restate what the code does.

## 6. Building and Testing

There is no `cargo test`; the kernel targets bare metal. Before opening a PR:

1. Confirm the project builds cleanly:
   ```
   python scripts/<target-arch>/build.py build
   ```
   This compiles the kernel and assembles a bootable ISO inside Docker. Incremental builds use MD5 hashing to skip Docker when nothing has changed.

2. Boot it in QEMU and verify nothing regressed:
   ```
   python scripts/<target-arch>/build.py run
   ```

See [README.md](README.md) for full setup instructions (Docker, QEMU, OVMF).

---

### Questions?

Start a discussion on GitHub.
