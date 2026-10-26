# Rules — Application layer (`src/02_application/`)

## Scope

Repository traits (dependency inversion boundary), stateless services, DTOs, domain→DTO mappers.

## Rules

1. **Repository traits in `*/traits.rs`** — all mockable (`automock`), `Send` bounds, every method returns `Result<T, ApplicationError>`.
2. **Dependency inversion** — the application defines the ports (traits); the infrastructure implements them. The application never imports `03_infrastructure`.
3. **Errors: `ApplicationError`** with `thiserror` — wraps `DomainError` via `#[from]`, adds `PackNotFound`, `InvalidManifest`, `Io` variants.
4. **Stateless services** — no mutable state; services receive `Box<dyn Trait>` dependencies.
5. **DTOs distinct from entities** — application mappers (`PackSummaryDto::from_pack`) convert domain→DTO. The UI only sees DTOs.
6. **Stateless mappers** — associated functions on unit structs; never state.
7. **No hidden I/O** — the application layer orchestrates; actual I/O lives behind repository traits.
8. **Tests with `mockall`** — `Mock*Repository` to mock the traits; co-located tests in the service files.

## Forbidden anti-patterns

- Importing `03_infrastructure` or any filesystem type.
- Returning a domain entity directly to the UI (always a DTO).
- Rendering or key-handling logic (that is the presentation's role).
- Mutable state in a service (use a UI-side presenter instead).
