---
name: rust-feature-ddd
description: Implement a new feature in this project while respecting the 4-layer DDD architecture (domain → application → infrastructure → presentation → wiring). Use whenever a task adds a behavior, a screen, an entity or a filesystem operation.
---

# Skill: DDD implementation of a Rust feature

## Project context

rusty-agentic-pack-manager is a Rust TUI (ratatui + crossterm + syntect; serde + toml manifests; **synchronous** std::fs, no tokio) to manage agentic packs.

**Strictly inward** dependency flow:

```
ui (04_presentation) → application (02_application) → domain (01_domain) ← infrastructure (03_infrastructure)
```

> Detailed per-layer rules: load `rules/domain.md`, `rules/application.md`, `rules/infrastructure.md`, `rules/presentation.md` at the repository root (shared source of truth — see also the `layer-rules` skill).

## Procedure — 5 steps

Layer order: **domain → application → infrastructure → presentation → wiring**. One step = one potential commit; the repo stays green at every step.

1. **Domain** (`src/01_domain/<area>/`)
2. **Application** (`src/02_application/<area>/`)
3. **Infrastructure** (`src/03_infrastructure/<area>/`)
4. **Presentation** (`src/04_presentation/tui/`)
5. **Wiring** (`src/main.rs`, `src/lib.rs`)

Detailed checklists per step: [references/layer-checklists.md](references/layer-checklists.md).

## Tests (mandatory)

- New logic → co-located UT; new user-facing behavior → IT scenario in `tests/`. See [rules/testing.md](../../../rules/testing.md) and the `rust-testing` skill.

## Verification (mandatory before delivery)

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo check
```

## Forbidden anti-patterns

- Importing a crate in `01_domain` (thiserror alone tolerated for errors).
- Using a `toml` or raw `std::fs` type outside `03_infrastructure` adapters.
- Exposing a domain entity to the UI (always a DTO).
- Business logic in adapters or UI action handlers.
- A new constructor that bypasses validation (infallible `pub fn new` on a validated value object).
