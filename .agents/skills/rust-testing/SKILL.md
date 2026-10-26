---
name: rust-testing
description: Write this project's Rust tests — co-located UT (mockall, tempdir) or IT in tests/ (TestBackend, real App). Use this skill whenever a task asks for tests or testable code is added.
---

# Skill: Rust testing

> **Source of truth for conventions**: [rules/testing.md](../../../rules/testing.md) (UT vs IT threshold, anti-patterns). This skill is the operational summary.

## Choose: UT or IT

| Situation | Where |
|---|---|
| Pure logic (invariant, mapper, mocked service, render function) | **UT** — `#[cfg(test)]` in the file |
| User-facing behavior (screen × key × state) | **IT** — scenario in `tests/` |
| Business logic bugfix | **UT** regression first (red), then fix |
| New visible behavior on screen | **IT** scenario |

## UT — per layer

### Domain (`01_domain`)
- Test the invariants: valid construction, rejection of invalid inputs (`new() → Err(DomainError::...)`).
- No mocks: the domain is pure.

### Application (`02_application`)
- `#[automock]` on the repository trait if not already there.
- Explicit expectations: `mock.expect_xxx().with(eq(...)).returning(|_| Ok(...))`.
- Cases: success, repository error propagated, domain → DTO mapping correct.

### Infrastructure (`03_infrastructure`)
- Test adapters on `tempfile::TempDir` fixtures — never against system paths.
- Manifest → entity conversions tested with hand-built fixtures.

### Presentation (`04_presentation`)
- Test the pure render functions (`split_view.rs`, `viewer/view.rs`) via `ratatui::backend::TestBackend`.
- Test the display-state logic (selection, scroll, popups) through `app.dispatch(&KeyEvent)` without a real terminal.

## IT — `tests/`

- Build the real `App` as in `main.rs` (real adapters on tempdir packs — no daemon, no network needed).
- Driving: `app.dispatch(&KeyEvent)`; rendering via `TestBackend`, assertions on the flattened buffer.
- Every new user-facing feature → an added scenario in the relevant `tests/` suite.

## Common conventions

- One test per behavior, descriptive name: `rejects_absolute_destination`, not `test_1`.
- Arrange / Act / Assert, one logical assertion per test.
- Bugfix: reproduction test first (red), then fix.

## Verification

```bash
cargo test                 # everything (UT + IT)
cargo test --lib           # UT only
cargo test --test <suite>  # one IT suite
cargo test <name>          # one targeted test
```

## Forbidden anti-patterns

- Tests against system paths or an external environment in the standard suite.
- `#[ignore]` without a written justification.
- Modifying a test to "make pass" a refactoring.
