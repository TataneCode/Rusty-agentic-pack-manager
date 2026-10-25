# Quality rules — cross-cutting (all layers)

Verification commands, **in this order**, before any delivery:

```bash
cargo fmt                  # format — no diff must remain
cargo clippy -- -D warnings # strict lint: CI treats warnings as errors
cargo test                 # all tests
cargo check                # fast compile (MSRV 1.88)
cargo build --release      # only if the delivery concerns the binary
```

Every delivery must pass `fmt` + `clippy -D warnings` + `test` without errors.

## Global style

- Edition 2021, **MSRV 1.88** (`rust-version` in `Cargo.toml`). No nightly features, no crate requiring a newer toolchain.
- **Errors: always `thiserror`** for layer errors; never `String` nor `anyhow` in expected layer errors. `anyhow` reserved for the `main.rs` edge.
- Error chain: `DomainError` → `ApplicationError` (`#[from]`) for the UI; `ApplicationError::Io` carries the failing path.
- **Synchronous application** — no tokio; all I/O via `std::fs` behind repository traits.
- **Memory safety**: avoid superfluous `clone()`, prefer references.
- **Naming**: Rust conventions (snake_case functions, PascalCase types); rename rather than add `#[allow(clippy::...)]`.
- **Dependencies**: the minimum necessary, the narrowest features possible.
- **Tests**: unit tests co-located in `#[cfg(test)]`, integration tests in `tests/`, `mockall` for traits, test name = behavior (e.g. `rejects_absolute_destination`); conventions in [testing.md](testing.md).
- **Minimal diff**: change only what is necessary; match the existing style; read a file entirely before modifying it.
