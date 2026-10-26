---
name: rust-dependency
description: Add or update a Cargo dependency in this project — MSRV-compatible version choice, minimal features, Cargo.lock verification. Use this skill as soon as a task touches Cargo.toml or introduces a crate.
---

# Skill: Cargo dependency management

## Systematic procedure — 5 steps

### 1. Justify the dependency
- Answer before adding: which problem cannot be solved with std or the existing crates?
- Check whether an already-present crate covers the need (e.g. `serde` for serialization, `toml` for manifests).

### 2. Project constraints
- MSRV **1.88**: verify the crate (and its version) supports this toolchain. The crate's `rust-version` ≤ 1.88.
- Edition 2021.
- The domain (`01_domain`) stays at **zero external crates** (thiserror alone tolerated for errors). Never a new dependency used by the domain.

### 3. Declare with minimal features
- List features explicitly (e.g. syntect with `default-fancy`, no onig; no blanket `full`).
- Constrained minor version (`"0.30"`, not `*` nor a needlessly wide `^`).
- Dev-dependencies separate (`mockall`, `tempfile`).

### 4. Lock and verify
```bash
cargo update -p <crate>       # or manual add + cargo build
cargo check                   # resolution + MSRV compile
cargo clippy -- -D warnings
cargo test
```
- `Cargo.lock` is committed: include it in the commit.
- Verify no existing crate's features were widened along the way (`cargo tree -e features` if in doubt).

### 5. Document in Cargo.toml
- Group by role with comments, like the existing style (`# TUI framework`, `# Syntax highlighting`, `# Serialization`, `# Error handling`...).

## Forbidden anti-patterns

- Adding a crate "just in case" or for a single trivial function implementable in a few lines.
- Widening features (implicit `default-features = true` + broad features) without necessity.
- Updating dependencies unrelated to the task in the same commit.
- Introducing a dependency that the domain uses or that raises the MSRV.
