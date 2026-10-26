---
name: rust-quality-gate
description: Rust quality gate — formatting, strict clippy, MSRV 1.88, this project's code style. Use this skill before any code delivery, on CI failure, or when a quality review is requested.
---

# Skill: Rust quality gate

## Verification commands (in this order)

```bash
cargo fmt                  # format — no diff must remain
cargo clippy -- -D warnings  # strict lint: CI treats warnings as errors
cargo test                 # all tests
cargo check                # fast compile (MSRV 1.88)
cargo build --release      # only if the delivery concerns the binary
```

Every delivery must pass `fmt` + `clippy -D warnings` + `test` without errors.

## Project style rules

> Shared source of truth: `rules/quality.md` and the per-layer rules (`rules/*.md`) at the repository root. This section is the operational summary.

- Edition 2021, MSRV **1.88** (declared in `Cargo.toml` via `rust-version`). No nightly features, no crates requiring a newer toolchain.
- Error handling: **always `thiserror`** for layer errors (never `String` nor `anyhow` in expected layer errors). `anyhow` reserved for the `main.rs` edge.
- Error chain: `DomainError` → `ApplicationError` (`#[from]`) for the UI; `ApplicationError::Io` carries the failing path.
- **Synchronous application** — no tokio; all I/O via `std::fs` behind repository traits.
- Memory safety: avoid superfluous `clone()`, prefer references.
- Naming: follow Rust conventions (snake_case functions, PascalCase types); rename rather than add `#[allow(clippy::...)]`.
- Dependencies: the minimum necessary, the narrowest features possible.

## CI failure — diagnosis

1. Read the first error message from the top (`cargo clippy` or `cargo test`), not the cascade errors.
2. Reproduce locally with the exact CI command.
3. Fix the demonstrated cause, not a guessed symptom.
4. Re-verify the full chain before pushing.
