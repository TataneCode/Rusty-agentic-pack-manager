# Rules — Domain layer (`src/01_domain/`)

## Scope

Entities (`Pack`, `PackItem`), value objects (`PackName`, `PackVersion`, `TargetCli`, `PackItemKind`), domain errors, business invariants.

## Rules

1. **Pure Rust, zero external crates** — no dependencies, not even `serde`. The domain compiles standalone.
2. **Errors: `thiserror` only** — `DomainError` for invariant violations. Never `String` nor `anyhow`.
3. **`with_*` builder pattern** for optional fields:
   ```rust
   Pack::new(name, version, description, clis, items)
   ```
4. **Validating constructors** — every entity or value object constructor validates its invariants (e.g. `PackName` rejects uppercase) and returns `Result<T, DomainError>` when the invariant can be violated.
5. **No I/O** — no network, file, or clock operations. The domain is deterministic.
6. **No application logic** — no orchestration, no DTOs (DTOs live in the application layer).
7. **Co-located tests** in `#[cfg(test)]` in each file, no mocks (the layer is pure).

## Forbidden anti-patterns

- Importing anything from `02_application`, `03_infrastructure` or `04_presentation`.
- Adding an external dependency to this layer.
- Mutable `pub` fields on entities: state changes through named methods.
- `unwrap()`/`expect()` outside tests.
