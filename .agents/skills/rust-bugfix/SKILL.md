---
name: rust-bugfix
description: Fix a bug in this project — reproduction, root-cause localization in the faulty layer, regression test. Use this skill as soon as incorrect behavior is reported or a test fails.
---

# Skill: Rust bug fixing

## Systematic procedure — 5 steps

### 1. Reproduce
- Identify the minimal command that triggers the bug (`cargo test <name>`, a precise TUI scenario).
- If the bug depends on the filesystem, isolate the faulty layer first via mapped tests (pure domain, mocked application) before concluding "infrastructure".
- If impossible to reproduce, ask for the exact steps rather than guess.

### 2. Locate the faulty layer (thanks to the DDD architecture)
| Symptom | Likely layer |
|---|---|
| Violated invariant, invalid data accepted | `01_domain` |
| Wrong DTO, incorrect service logic | `02_application` |
| Badly mapped IO error, wrong manifest conversion | `03_infrastructure` |
| Rendering, selection, keys, screen state | `04_presentation` |

The fix happens **in the faulty layer**, never by compensating in a neighboring layer.

### 3. Write the regression test first
- See the `rust-testing` skill: co-located test in the layer's file, name describing the bug (`rejects_absolute_destination`).
- Verify it fails before the fix (red), otherwise it tests nothing.

### 4. Fix at the smallest scope
- Minimal change in the identified layer.
- Do not "improve" adjacent code during a bugfix.
- If the fix requires a refactoring, decouple it: bugfix first, refactor after.

### 5. Verify the full chain

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Forbidden anti-patterns

- Fixing the symptom in the UI while the cause is below (e.g. catching an error earlier to mask it).
- Adding a convenience `unwrap()`/`expect()` to "unblock".
- Pushing without a regression test for a business-logic bug.
