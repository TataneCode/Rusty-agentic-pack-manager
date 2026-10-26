---
name: rust-architecture-planning
description: Produce an architecture or design plan for this project — framing, audit of the existing, options, decision, step-by-step implementation plan. Use whenever planning, design, a new functional area, or any non-trivial implementation is discussed.
---

# Skill: Rust design and planning

## Project context

rusty-agentic-pack-manager: Rust TUI (ratatui, crossterm, syntect; serde + toml manifests) to manage agentic packs. **Synchronous** application (std::fs, no tokio). DDD 4 layers, strictly inward flow:

```
04_presentation → 02_application → 01_domain ← 03_infrastructure
```

Directories: `src/01_domain/`, `src/02_application/`, `src/03_infrastructure/`, `src/04_presentation/tui/`. Context and navigation: [CONTEXT.md](../../../CONTEXT.md). Layer rules (source of truth): [rules/](../../../rules/).

## Procedure — 6 steps

### 1. Frame the request
Restate as "<verb> <scope> for <benefit>"; list the non-negotiable constraints (DDD, MSRV 1.88, no new dependency without justification, clippy `-D warnings`).

### 2. Audit the existing
Locate the relevant areas (`pack`, `pack_view`, `explorer`); identify the impacted symbols (`grep -rn`) across the 4 layers; check for precedents — a problem already solved elsewhere is a pattern to reuse, not reinvent.

### 3. Propose options (2-3 max)
Short description, per-layer impacts, estimated cost, risks. Recommend one option with justification — never hollow options.

### 4. Decide the architecture points
- Placement: the code goes in the innermost layer concerned.
- New entity → value objects + dedicated `DomainError` variants.
- New data source → repository trait in `02_application`, adapter in `03_infrastructure`.
- New screen → display state in `app.rs` + pure view functions + action handlers in `router.rs` + `AppAction` variants + `main.rs` wiring (state/views/actions live in app.rs/split_view+viewer/router.rs, not a per-panel file triad — see `rules/presentation.md`).
- Every decision noted with its rationale.

### 5. Produce the implementation plan
Format, step order and verification commands: see [references/plan-format.md](references/plan-format.md).

### 6. Final plan validation
Checklist (no code, consistency, anticipated tests): see [references/plan-format.md](references/plan-format.md).

## Forbidden anti-patterns

- A plan mixed with detailed implementation (the plan describes the what/why, not the line-by-line how).
- An architecture decision without rationale.
- A step too broad ("refactor the UI") or that leaves the repo broken along the way.
- Ignoring the project's existing patterns in favor of a generic approach.
