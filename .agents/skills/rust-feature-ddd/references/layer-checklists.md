# Reference — Implementation checklists per layer

Detail of `rust-feature-ddd` (steps 1 to 5). Loaded on demand, layer by layer. The full normative rules live in `rules/` (source of truth) — this file is only an operational memory aid.

## 1. Domain (`src/01_domain/<area>/`)

- Entity / value object in pure Rust: **zero external crates** (thiserror tolerated for errors).
- Value objects: private fields + `new() -> Result<Self, DomainError>` (never an infallible constructor for validated data).
- Invariants in the type, not in the upper layers.
- Errors: variants in the domain error enum (`src/01_domain/pack/error.rs`) with `Display`.
- Unit tests in `#[cfg(test)]` in the same file.

## 2. Application (`src/02_application/<area>/`)

- Repository trait if a new capability: `#[automock]`, returns `Result<T, ApplicationError>`, `Send` bounds.
- Stateless service: `pub struct XxxService { repo: Box<dyn XxxRepository> }`.
- Dedicated DTO + `domain ↔ DTO` mapper (associated functions on unit structs). Never a domain entity exposed to the UI.
- Errors: extend `ApplicationError` in `src/02_application/error.rs`.

## 3. Infrastructure (`src/03_infrastructure/<area>/`)

- Adapter implementing the application trait; manifest → entity and DTO conversions via dedicated mappers.
- `std::io::Error` → `ApplicationError::Io { path, source }` carrying the failing path.
- Tempdir-based tests; no hardcoded system paths; existing destinations never silently overwritten.

## 4. Presentation (`src/04_presentation/tui/`)

- Separation of the three concerns (structure of this project, per `rules/presentation.md`):
  - `app.rs` — mutable display state (selection, scroll, focus, popups)
  - `split_view.rs` / `viewer/view.rs` — pure render functions (references → widgets)
  - `router.rs` — action handlers, thin wrappers around application services
- Overlays (`popup`, `input`, `confirm`) evaluated before the panel handlers.
- Keys mapped in `common/keys.rs` → `AppAction` variants per `KeyContext`.
- The UI catches `ApplicationError` and shows it as a popup message; never panics.

## 5. Wiring (`src/main.rs` and `src/lib.rs`)

- Manual bottom-up wiring, no DI framework:
  `Fs*Adapter → XxxService → App::new(...)`
- Expose the modules in the relevant `mod.rs` / `lib.rs`.
