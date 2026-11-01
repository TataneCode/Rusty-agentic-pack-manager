# Plan: E03 — CLI headless : install / uninstall / list / show

> Status: **approved** — branch `vibe/e03-cli-headless-33460d`; gate to verify before delivery (`fmt`, `clippy --all-targets -- -D warnings`, `cargo test`). Source: `.evolutions/e03-cli-headless.md` via skills `/challenge` then `/plan` (architect). Format: `rust-architecture-planning`. User decisions are marked ✅.

## Challenge (framing, per `/challenge`)

### Restatement

Install/uninstall/list/show exist only in the TUI; only `rapm pack create` is scriptable. E03 exposes the existing `PackService` operations as headless CLI subcommands with the same rules (destination validation, `packs_index.toml`, exit codes 0/1/2), so harness engineering (scripts, CI, test containers) can drive rapm without a terminal.

### Blocking ambiguities (settled by decision, marked ✅)

1. **`rapm show <pack>` output** ✅ Summary only: manifest fields (name, version, description, CLIs) + one line per item (kind, source, per-CLI destinations). No file-content dump (that would wire `PackViewService`; left to a later evolution), no `--json`.
2. **`--dest` default** — settled by convention: current directory (same as the TUI `start_dir`); `--cli` default = `PackService::default_cli` (first declared CLI, fallback `vibe`). The E07 selector stays out of scope.

### Detected scope creep (excluded)

- `--dry-run` (E04), `pack try` (E06), default-CLI selector change (E07), user scope (E09), `--json` output, `show <pack> <path>`.

### Proposed scope

- **Minimal = full**: additive parser in `src/presentation/cli.rs`, DI of `PackService` in `main.rs`, IT via `CARGO_BIN_EXE_rapm` on tempdir, docs, gate vert.

### Decision requested

✅ Go on the scope above.

---

## Goal

```bash
rapm list [--cli <cli>] [--installed [--dest <dir>]]
rapm show <pack>
rapm install <pack> --dest <dir> [--cli <cli>] [--force]
rapm uninstall <pack> --dest <dir> [--cli <cli>]
```

served by the existing `PackService` (plus one detail DTO for `show`), exit codes unchanged: `0` success, `1` runtime error, `2` usage error.

## Architecture decisions

- **D1: no new domain/application logic** — `PackService::{list, installed_in, default_cli, install, uninstall, find}` already carries the rules (plan validation, collision → `DestinationExists`, index record/remove). The gap is presentation parsing + DI wiring (`main.rs` builds only `PackCreationService` on the CLI path today).
- **D2: additive parser in `src/presentation/cli.rs`** — new `CliCommand` variants `List(ListOptions)`, `Show { pack }`, `Install(InstallOptions)`, `Uninstall(UninstallOptions)` reusing the existing `TokenParser` (`--flag value` and `--flag=value`). `--dest` defaults to the cwd; `--cli` defaults via `PackService::default_cli`. Exit-code mapping in `main.rs` is untouched.
- **D3: `show` = summary only** — a `PackDetailDto` in `application/pack/dto.rs` (name, version, description, clis, items as plain strings) built from `PackService::find`; printed as plain text. Presentation never imports the domain (architecture test enforces it), so the DTO is required.
- **D4: output shape** — `list`: one line per pack (`name  version  clis  N item(s)`); `list --installed`: one line per index entry (`name  cli  path, ...`); `show`: header fields + one line per item (`kind  source  cli=dest`); `install`/`uninstall`: one confirmation line with counts. stdout for data, stderr for errors (existing `main.rs` path).
- **D5: DI in `main.rs`** — a `build_pack_service(&library)` helper building `PackService` exactly like `build_app` (FsPackRepository + profiles, FsInstallDestination, FsInstallIndexRepository, CliProfileService), passed to `cli::run` next to the creation service.
- **D6: IT via `env!("CARGO_BIN_EXE_rapm")`** in `tests/cli_headless.rs`: spawn the real binary with `HOME`/`XDG_DATA_HOME` on a tempdir and cwd = temp target repo, covering the fiche's acceptance criteria (install copies files + writes `packs_index.toml`, exit 0; reinstall without `--force` → exit 1; `list`/`show` read-only without a TTY; usage error → exit 2).

## Steps

1. **Detail DTO + accessor** — `PackDetailDto`/`PackItemDto` in `src/application/pack/dto.rs`, `PackService::detail(name)` in `src/application/pack/service.rs` (+ UT) — files: `src/application/pack/dto.rs`, `src/application/pack/service.rs` — verification: `cargo test --lib application::pack`.
2. **Parser + handlers** — new subcommands in `src/presentation/cli.rs`, USAGE update, `run(args, creation, pack)` signature, co-located UT for parsing — files: `src/presentation/cli.rs` — verification: `cargo test --lib presentation::cli`.
3. **Wiring** — `build_pack_service` in `src/main.rs`, CLI branch passes it to `cli::run`; `--dest` default = cwd resolved in the handler — files: `src/main.rs` — verification: `cargo check`.
4. **IT** — `tests/cli_headless.rs` spawning `CARGO_BIN_EXE_rapm` on tempdir (acceptance criteria + exit codes 0/1/2) — verification: `cargo test --test cli_headless`.
5. **Docs** — MANUAL.md (new "Headless CLI" section) + README.md mention — verification: `cargo check`.
6. **Gate + registre** — `cargo fmt` → `cargo clippy --all-targets -- -D warnings` → `cargo test` → `cargo check`; `.evolutions/index.md`: E03 → `en cours` with the plan link, `livré` at delivery (PR link).

## Risks and mitigations

- **Binary-spawn IT depends on env isolation** → set `HOME`, `XDG_DATA_HOME` and cwd explicitly per spawn so the user library lands in the tempdir.
- **Default `--dest` = cwd could surprise** → documented in USAGE and MANUAL.md.
- **Reinstall collision must exit 1, not 2** → `DestinationExists` is a runtime `ApplicationError`, already mapped to 1 in `main.rs`.
- **`show` on an unknown pack** → `PackNotFound` runtime error → exit 1.

## Out of scope

`--dry-run` (E04), `pack try` (E06), CLI selector at install (E07), user scope (E09), `--json`, `show <pack> <path>` file dump, any change to `PackService` semantics.
