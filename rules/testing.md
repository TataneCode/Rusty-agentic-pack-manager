# Testing rules — UT & IT (cross-cutting, all layers)

**Single source of truth** for test conventions. The user-facing scenario matrix (what is covered where) lives in `docs/`. The operational skill is `rust-testing` ([.agents/skills/rust-testing/](../.agents/skills/rust-testing/)).

## Decision threshold: UT or IT?

| Situation | Where | Why |
|---|---|---|
| Pure logic (invariant, mapper, service with mock, presenter, view function) | **UT** — co-located `#[cfg(test)]` in the file | Fast, isolated, no terminal needed |
| User-facing behavior crossing layers (screen × key × state) | **IT** — scenario in `tests/` | Validates the real wiring, not a copy |
| Business logic bugfix | **UT** regression first (red), then fix | The regression is fixed at the faulty layer's level |
| New visible behavior on screen | **IT** + entry in the scenario matrix | Guarantees the user contract coverage |

Rule: **1 behavior = 1 test**, named after the behavior (`rejects_absolute_destination`, not `test_1`). Arrange / Act / Assert.

## UT — conventions

- Co-located in `#[cfg(test)]` **in each file** — no parallel `tests/` directory for unit tests.
- **Domain**: no mocks (pure layer) — test invariants, `new()` rejections.
- **Application**: `#[automock]` on repository traits; explicit expectations (`mock.expect_xxx().with(eq(...)).returning(...)`); co-located tests in service files.
- **Infrastructure**: test adapters on `tempfile::TempDir` fixtures. **Never** against system paths.
- **Presentation**: test the pure functions of `view.rs` (references → widgets) and `presenter.rs` logic (selection, scroll) without heavy mocks.
- If a bug is fixed: write the failing test first (reproduction), then fix.

## IT — conventions

- Build the real `App` as in `main.rs` — real adapters on tempdir packs; rendering and action handling need no daemon or network.
- Driving: `app.dispatch(&KeyEvent)` (rendering via `ratatui::backend::TestBackend`, assertions on the flattened buffer).
- **Every new user-facing feature** adds a scenario to the integration suites.

## Verification

```bash
cargo test                 # everything (UT + IT)
cargo test --lib           # UT only
cargo test --test <suite>  # one IT suite
cargo test <name>          # one targeted test
```

## Forbidden anti-patterns

- Testing against system paths or an external environment in the standard suite.
- `#[ignore]` without a written justification.
- Modifying a test to "make pass" a refactoring (a refactoring does not change tests).
