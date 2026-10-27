# Plan: install tracking (`packs_index.toml`), Installed panel, uninstall

> Status: **implemented** — install tracing + Installed panel (r2) + uninstall (`U`). Source: skills `/challenge` then `/plan`. User decisions are marked ✅.

## Goal

Trace every pack installation in a `packs_index.toml` index at the root of the target repository (one entry per `(pack, CLI)` with the relative paths actually copied), show the installations of the explorer's current directory in a new **Installed** panel below the packs panel, hide installed packs from the available list, and uninstall (`U`) a pack by removing exactly the recorded assets.

## Framing context (challenge)

- Restatement: track installations per target repository, display them in the TUI, and make uninstall possible by removing every asset a pack installed.
- User decisions:
  - ✅ **Scope = index + panel + uninstall** in this iteration (undeploy was listed "out of scope v1" in CONTEXT.md; the user explicitly pulled it in).
  - ✅ **Layout**: two columns; the left column is split horizontally — r1 = available packs, r2 = installed packs. r2 is interactive. A pack present in r2 no longer appears in r1.
  - ✅ **Index granularity**: one entry per `(pack, cli)`: `name`, `version`, `cli`, `paths` (relative paths actually copied). Reinstall = upsert of the entry.
- Accepted limits (documented in MANUAL.md): no rollback on partial install failure (index written only after a successful install); manually deleting `packs_index.toml` loses the tracking.

## Architecture decisions

- **D1: Domain `InstalledPack`** (`src/domain/pack/installed.rs`) — entity with `PackName`, `PackVersion`, `TargetCli` and non-empty `paths`; every path must be relative and stay inside the target root (no absolute, no `..`). New `DomainError` variants: `EmptyInstallPaths`, `InstallPathOutsideTarget`, `ReservedDestination`. Pure layer, no serde.
- **D2: `plan_install` reserves `packs_index.toml`** — an item destination resolving to `<target_root>/packs_index.toml` is rejected with `ReservedDestination` (the index belongs to the application, never to a pack).
- **D3: Application port `InstallIndexRepository`** (`#[automock]`) — `load(root)` (missing file = empty list), `record(root, entry)` (upsert by name+cli), `remove(root, name, cli)`. New `ApplicationError::InstallNotFound`.
- **D4: `PackService` gains a 3rd dependency** (`Box<dyn InstallIndexRepository>`). `install()` records the entry **after** the copies succeed. New methods: `installed_in(root) -> Vec<InstalledPackDto>` and `uninstall(name, cli, root) -> UninstallResultDto`. `InstallDestination` gains `remove(path)`.
- **D5: Uninstall order = assets first, then index entry** — a failing removal keeps the entry (retry possible); assets already missing on disk are tolerated (idempotent uninstall, drift-friendly).
- **D6: Infrastructure** — `FsInstallIndexRepository` (`src/infrastructure/pack/index.rs`, serde/toml, missing file = empty, corrupt file = `InvalidManifest`, entries validated through the domain entity so an escaping path is rejected at load) and `FsInstallDestination::remove` (file or recursive dir; missing path tolerated).
- **D7: Presentation state** (`app.rs`) — `Focus::Installed`, `installed: SelectList<InstalledPackDto>`, `refresh_installed()` called at startup, on `refresh_dir()`, after install and uninstall. r1 is filtered: a pack is unavailable once the index records it for its **first** target CLI (the CLI `A` installs for). `ConfirmDialog` gains a `kind: ConfirmKind` (`Delete` | `Uninstall`).
- **D8: Presentation view** (`split_view.rs`) — left column = `Layout::vertical` 60/40 (Packs / Installed); r2 titled `Installed`, one line per entry: `name vX — CLI label (N item(s))`, empty state ` None installed`.
- **D9: Keys/router** — `KeyContext::InstalledPanel`, `AppAction::Uninstall` (`u`), Tab cycles Packs → Installed → Explorer → Packs; `U` opens the confirm dialog ("Uninstall 'tdd' (vibe) — remove N item(s)?"). `action_label` maps the new action.
- **D10: Help & docs** — `installed_help()`; MANUAL.md (keys, index format, uninstall semantics), README.md, CONTEXT.md updated; this plan archived.

## Steps

1. Domain — `InstalledPack` + errors + reservation of `packs_index.toml` in `plan_install` + UT — `cargo test --lib domain`
2. Application — `InstalledPackDto`, `UninstallResultDto`, `InstallIndexRepository`, `InstallDestination::remove`, `PackService` (record/uninstall/installed_in) + UT mockall — `cargo test --lib application`
3. Infrastructure — `FsInstallIndexRepository` + `FsInstallDestination::remove` + UT tempdir — `cargo test --lib infrastructure`
4. Presentation state — `Focus::Installed`, installed list, `refresh_installed`, r1 filter + UT — `cargo check`
5. Presentation view — r1/r2 split of the left column — `cargo check`
6. Presentation keys/router — `InstalledPanel` context, `Uninstall` action, 3-zone Tab cycle, confirm dialog — `cargo test --lib presentation`
7. Wiring — `main.rs` + 3-arg `PackService::new` in existing ITs — `cargo check --tests`
8. IT — `tests/pack_install.rs` (index written with relative paths, upsert, uninstall end-to-end, failed install leaves no index change) + `tests/split_screen_ui.rs` (r2 listing, r1 exclusion, Tab cycle 3 zones, `U`+confirm, cancel, help) — `cargo test`
9. Docs — MANUAL.md, README.md, CONTEXT.md, this plan — `cargo fmt --check`
10. Quality gate — `cargo fmt` → `cargo clippy --all-targets -- -D warnings` → `cargo test` → `cargo check`

## Risks and mitigations

- **Drift** (asset deleted by hand between install and uninstall) → uninstall tolerates missing assets (D5); documented in MANUAL.md.
- **Partial install failure** → index written only after all copies succeed; assets may be orphaned, documented as an accepted limit (no transactional rollback in the synchronous app).
- **Index deleted by hand via the explorer (`D`)** → tracking lost, uninstall impossible from the TUI; documented; protection is out of scope for this iteration.
- **Uninstall safety** → recorded paths are validated by the domain entity (relative, no `..`) and resolved against the target root before any filesystem call.
- **Tab cycle 2 → 3 states** → existing `tab_toggles_focus` IT updated to the 3-zone cycle.

## Out of scope

- Timestamps, integrity hashes, orphan-entry cleanup.
- Protecting `packs_index.toml` against deletion via the explorer.
- CLI picker at install time (unchanged: the pack's first declared CLI).
- Transactional rollback of a failed install.
- 7z/tar packs, CLIs other than vibe/gemini (unchanged).
