# Rules — Presentation layer (`src/04_presentation/`)

## Scope

Ratatui rendering, key handling, screen state machine, `main.rs` (DI wiring).

## Rules

1. **Separation of state, rendering and actions**: `app.rs` holds the mutable display state (selection, scroll, focus/mode), `split_view.rs` / `viewer/view.rs` are pure render functions (take references, produce widgets), `router.rs` holds the action handlers (thin wrappers around application services). A per-panel presenter/view/actions file layout is not required; keep the three concerns separated wherever they live.
2. **The UI only sees DTOs** — never a direct domain entity; actions consume application services.
3. **State machine**: `app.rs` owns `Focus`/`Mode` enums that drive rendering and keys. Overlay states (`popup`, `input`) are evaluated **before** panel handlers.
4. **Key mapping**: `common/keys.rs` maps raw `KeyEvent` to `AppAction` variants per `KeyContext`. No scattered `match` on KeyCode.
5. **Errors: catch `ApplicationError`** → popup message. Never panic on the UI side.
6. **Contextual help** — the spacebar popup content depends on focus/mode (`common/help.rs`).
7. **No business logic** — a presenter manages display; business decisions live in the application/domain.

## Forbidden anti-patterns

- Importing `01_domain` directly or `03_infrastructure` (always via the application).
- Direct I/O (filesystem calls) in a view or presenter.
- Stateful views — `view.rs` stays pure functions.
- Key handling outside `keys.rs`/dedicated handlers.
