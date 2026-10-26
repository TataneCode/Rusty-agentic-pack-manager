---
name: rust-refactoring
description: Refactor this project's code (splitting large files, moving types, renames) while preserving behavior and the DDD architecture. Use this skill for any refactoring task, module extraction or code cleanup.
---

# Skill: Rust refactoring

## Project context

DDD 4-layer architecture (`01_domain` → `02_application` → `03_infrastructure` → `04_presentation`), MSRV 1.88, strict CI (`clippy -D warnings`). Repository history: notable cleanups already done = removal of the empty per-panel triad stubs (state/views/actions live in app.rs / split_view+viewer / router.rs — see `rules/presentation.md`), dead-code deletions, dedicated error variants.

## Systematic procedure — 6 steps

1. **Establish the baseline**: `cargo test && cargo clippy -- -D warnings` before any change. If not green, stop and flag it.
2. **Map the usages**: `grep -rn "<symbol>" src/` for every moved symbol. List all call sites before touching code.
3. **Small atomic steps**: one refactor = one commit. Typical order:
   - Extraction: create the new module → move the code → temporarily re-export if useful (`pub use`) → update the `mod.rs` → remove the temporary re-exports.
   - Cross-layer move: the type goes to the innermost layer concerned (an application DTO never lives in a global `shared.rs`).
   - Rename: `cargo clippy` guides toward now-useless `#[allow]` to delete (e.g. removing `allow(clippy::enum_variant_names)` after renaming variants).
4. **Preserve public APIs**: check `src/lib.rs` — it is the public boundary; anything not re-exported can be renamed freely.
5. **Verify at every step**: quick `cargo check` after each move, full suite (`fmt`, `clippy -D warnings`, `test`) at the end.
6. **Document**: if the refactor is significant, add a dated note in `docs/plans/` listing the before/after files.

## Final verification

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

No test may be modified to "make pass": if a test changes, it is a behavior change, not a refactoring.

## Forbidden anti-patterns

- Mixing refactor and functional change in the same commit.
- Deleting or weakening a test to make it green.
- Creating catch-all "utils" files; place types in their DDD layer.
- Breaking the dependency-flow rule during a move.
