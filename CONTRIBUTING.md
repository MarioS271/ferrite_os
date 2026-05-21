# Contributing to FerriteOS

Thank you for your interest in contributing! To keep the project safe and maintainable, please follow these guidelines.

## 1. Workflow

1. **Fork**: Create a fork of the repository in your own account.
2. **Branch**: Create a new branch for your changes (`git switch -c feature/my-feature`).
3. **Commit**: Write clear and descriptive commit messages.
4. **Push**: Push the branch to your fork.
5. **Pull Request (PR)**: Open a PR against the `main` branch.

## 2. Pull Request Rules

- Briefly describe what you changed or added and why.
- Keep PRs small and focused on a single topic.
- **Never** commit `build.toml`, OVMF firmware files, or any other private configuration.
- Do not remove entries from or add unnecessary entries to ignore files (`.gitignore`, `.claudeignore`) unless absolutely necessary.

## 3. Code Style

- All kernel code is written in Rust; follow standard Rust naming conventions (`snake_case` for variables, functions, modules, and files).
- Keep `unsafe` blocks minimal and always document why they are necessary.
- For `unsafe` blocks with inline assembly, do not document *why* the unsafe is there, rather document why the inline asm is needed.
- For documenting `unsafe` and `asm!` blocks, use `//` as comment prefix (do not use `///` or `//!`)
- Do not use comments to describe an obvious piece of code. Only use them to describe complex code.
- For categorizing a flow (like the kernel init in main.rs) you may use comments to create a distinction between code sections.
- Maintain the existing module and namespace structure (code goes under `src`, kernel stuff under `kernel`, …).

## 4. AI-Generated Code

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

## 5. Content

- This is a bare-metal OS kernel. Only contribute things that meaningfully advance the kernel itself (drivers, memory management, scheduling, etc.).
- Do not add new dependencies (`Cargo.toml`) without prior discussion. The `x86_64-unknown-none` target imposes strict constraints on what crates are usable.
- **Never** modify the linker script (`linker.ld`), the Limine entry point, or the memory layout without prior approval, as these are load-bearing for the entire boot process.
- Custom features that only serve a personal use case belong in a fork, not the upstream repo.

## 6. Building and Testing

There is no `cargo test`; the kernel targets bare metal. Before opening a PR:

1. Confirm the project builds cleanly:
   ```
   python run/build-*.py build
   ```
   This compiles the kernel and assembles a bootable ISO inside Docker. Incremental builds use MD5 hashing to skip Docker when nothing has changed.

2. Boot it in QEMU and verify nothing regressed:
   ```
   python run/build-*.py run
   ```

You can also combine both steps in one go:
```
python run/build-*.py all
```

To clean all build artefacts:
```
python run/build-*.py clean
```

See `README.md` for full setup instructions (Docker, QEMU, OVMF).

---

### Questions?

Open an issue or start a discussion on GitHub.