# Plan: E04 — Dry-run / plan d'installation (CLI `--dry-run` + preview TUI `P`)

> Status: **approved** — branch `vibe/e04-dry-run-271715`; gate to verify before delivery (`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`). Source: `.evolutions/e04-dry-run.md` via skills `/challenge` then `/plan` (architect). Format: `rust-architecture-planning`. User decisions are marked ✅.

## Challenge (framing, per `/challenge`)

### Restatement

`A` (TUI) and `rapm install` (E03) copy directly and refuse when a destination exists; E04 adds an install plan computable **without writing**: `--dry-run` on the CLI (per-entry status, dedicated exit code) and a `P` preview in the TUI before `A`.

### Blocking ambiguities (settled by decision, marked ✅)

1. **Status semantics** ✅ — `collision` = destination already on disk **and** recorded in `packs_index.toml` (owned by an installed pack); `hors index` = on disk but **not** tracked by the index (a user file); `ok` = free. Consequence: the plan reads the index read-only. Note: two "collisions" coexist — an in-pack duplicate destination (`DomainError::InstallCollision`) stays a **hard domain error** even in dry-run; only the `ok`/`collision`/`hors index` statuses reflect disk state.
2. **Exit codes** ✅ — exit 3 is reserved for dry-run when at least one entry is not `ok` (existing destination) **without** `--force`; `--dry-run --force` → exit 0 with statuses unchanged; a real install on an existing destination stays exit 1 (E03 unchanged).
3. **Scope** ✅ — full: CLI `--dry-run` + TUI `P` in the same delivery, sharing one DTO.

### Detected scope creep (excluded)

- Conflict strategies (`skip`/`overwrite`/`suffix`) = E05 — but the status enum is extensible so E05's `skipped` fits later.
- `--json`, a standalone `rapm plan` subcommand, the install-time CLI selector (E07 — `P` stays free on the packs panel), moving `plan_install` out of `error.rs`.

### Proposed scope

- **Minimal = full** (per ✅ decision 3): application plan DTO + service method, CLI flag with exit 3, TUI `P` preview popup sharing the same DTO, IT on tempdir/TestBackend, docs, green gate.

---

## Goal

```bash
rapm install <pack> --dest <dir> [--cli <cli>] [--force] --dry-run
```

prints one line per entry (`<status>  <source> -> <destination>`, status in `ok` / `collision` / `hors index`) without writing anything; exit 3 if any destination already exists and `--force` is absent, 0 otherwise. In the TUI, `P` on the packs panel opens the plan preview; `A` stays the direct install.

## Architecture decisions

- **D1: the domain stays pure and strict** — `plan_install` (`src/domain/pack/error.rs`) keeps its semantics (hard error on in-pack duplicate / invalid destination). Disk/index statuses are not domain knowledge (domain = no I/O, `rules/domain.md`): the fiche's `InstallPlan { entries }` lands as an **application DTO** (`InstallPlanDto`/`PlanEntryDto`/`PlanEntryStatus`), not a domain type.
- **D2: statuses** (✅) — `PlanEntryStatus::{Ok, Collision, Untracked}` with `as_str()` → `ok` / `collision` / `hors index`; extensible for E05.
- **D3: application** — `PackService::plan_install(pack_name, cli, target_root) -> Result<InstallPlanDto, ApplicationError>`: resolves pack + CLI profiles (like `install`), computes the domain plan, then classifies each entry via `InstallDestination::exists` + comparison of the root-relative path against every `paths` of `InstallIndexRepository::load` (read-only). **No new port.** `install` and `plan_install` share the same resolution (pack → cli → plan) so CLI, TUI and install always see the same entries.
- **D4: CLI** — `--dry-run` flag on `install` only (the shared `parse_pack_target` already gates `--force` per subcommand; same mechanism for `--dry-run`). Handler prints the plan and returns `Ok(3)` if any entry is not `ok` and `!force`, else `Ok(0)` — `main.rs` already exits with the code returned by `cli::run`. USAGE updated (option + exit-code line).
- **D5: TUI** — `AppAction::PlanPreview` mapped to `KeyCode::Char('P')` on `KeyContext::PacksPanel` only (`common/keys.rs`); the router computes the plan and sets `Popup::Plan(InstallPlanDto)`; `split_view.rs::render_popups` renders the list (title " Plan "), any key closes (existing popup priority); `A` unchanged. `help::packs_help()` gains a `P` line.
- **D6: DTO shape** — `InstallPlanDto { pack_name, cli, target_root, entries: Vec<PlanEntryDto> }`; `PlanEntryDto { source, destination (relative to the target root for display), status }`. The presentation only sees DTOs (E01).
- **D7: tests** — UT application (mockall) for the three statuses + "plan writes nothing"; UT CLI for parsing (`--dry-run` accepted on install, rejected on uninstall) and exit 0/3 via a small pure helper; IT `tests/cli_dry_run.rs` (real binary on tempdir: nothing written, no index, exit 0/3) + a `P` scenario in `tests/split_screen_ui.rs` (TestBackend buffer).
- **D8: docs/registry** — `MANUAL.md` (headless CLI section, keys table, exit codes), `.evolutions/index.md` (E04 → `planifié` with plan link, then `en cours` + PR link, `livré` at merge).

## Steps

1. **Plan + registry** — `docs/plans/e04-dry-run.md` (this file) + E04 → `planifié` in `.evolutions/index.md` — files: `docs/plans/e04-dry-run.md`, `.evolutions/index.md`.
2. **Application** — DTOs in `src/application/pack/dto.rs`, `PackService::plan_install` + shared resolution with `install` in `src/application/pack/service.rs`, co-located mockall UT — verification: `cargo test --lib application::pack`.
3. **CLI** — `--dry-run` in `InstallOptions`/parser/handler + USAGE in `src/presentation/cli.rs`, co-located UT — verification: `cargo test --lib presentation::cli`.
4. **TUI** — `AppAction::PlanPreview`, `P` mapping in `common/keys.rs`, handler in `router.rs`, `Popup::Plan` in `app.rs`, rendering in `split_view.rs`, help line in `common/help.rs`, UT keys — verification: `cargo test --lib presentation`.
5. **IT** — `tests/cli_dry_run.rs` (tempdir, `CARGO_BIN_EXE_rapm`, exit 0/3, no write verified) + `P` scenario in `tests/split_screen_ui.rs` — verification: `cargo test --test cli_dry_run --test split_screen_ui`.
6. **Docs** — `MANUAL.md` (+ README if relevant) — verification: `cargo check`.
7. **Gate + delivery** — `cargo fmt` → `cargo clippy --all-targets -- -D warnings` → `cargo test` → branch `vibe/e04-dry-run-271715`, commits, push, draft PR; index.md → `en cours` with PR link.

## Risks and mitigations

- **"Same entries" CLI/TUI/install** → shared DTO + shared resolution (D3/D6); verified in IT.
- **Dry-run writes nothing** → only `exists`/`load` (read-only); IT on tempdir asserts no files and no index.
- **Exit 3 vs 1 vs 2 confusion** → 3 = dry-run only, returned via `Ok(3)`; documented in USAGE + MANUAL; real install unchanged (1).
- **E05 extensibility** → `PlanEntryStatus` enum with `as_str()`; `Skipped` will slot in without breaking the DTO.
- **Long plan popup** → simple popup (any key closes), like Help/Error/Success; viewer-style scrolling not needed in v1.

## Out of scope

Conflict strategies (E05), `pack try` (E06), install-time CLI selector (E07), user scope (E09), drift (E10), `--json`, standalone `rapm plan`, moving `plan_install` out of `error.rs`, any change to `install` semantics.
