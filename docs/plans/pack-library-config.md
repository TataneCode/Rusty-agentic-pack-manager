# Plan: configurable pack library path (`rapm config`) + TUI startup guard

> Status: **draft** — awaiting user approval. Source: review of PR #5 (comment) + user request for a configuration file controlling the packs location. Format: `challenge` framing embedded below, then `rust-architecture-planning`.

## Challenge (framing, per `/challenge`)

### Restatement

The repo ships default packs and lets users add packs via `rapm pack create`, but the packs location is derived only from the environment (`$XDG_DATA_HOME/rapm/packs`, fallback `~/.local/share/rapm/packs`, `%APPDATA%\rapm\packs`). The user wants a configuration file inside the app to choose where the pack library lives, a CLI to set it, and a TUI startup guard when it is not configured.

### Blocking ambiguities (settled by decision, marked ✅)

1. **Config file or not, given XDG already allows relocation?** ✅ Decided: a config file IS wanted, because it unlocks more than the writable root — read-only extra roots (`extra_roots`, e.g. a team-shared library) — and gives Windows zip users the same mechanism (`%APPDATA%\rapm\config.toml`) instead of a hardcoded default.
2. **"TUI closes if the path is not configured"?** ✅ Reframed (challenge accepted): the TUI only closes when **no pack is readable at all**; if system packs exist (RPM/DEB), the TUI must open, show them, and display a hint that `pack create` needs a configured library. Closing the TUI otherwise breaks the packaged installs.
3. **rpm/deb asking for the path at install time?** ❌ Rejected in review: packages install as root, possibly multi-user or in CI; writing into `$HOME` at install time is wrong. The package keeps shipping system packs in `/usr/share/rusty-agentic-pack-manager/packs`; the library path stays a per-user setting via the config file. Not asked via debconf/rpm scriptlets.

### Detected scope creep (excluded)

- Asking the path during rpm/deb installation (rejected above).
- A TUI screen to edit the config (out of scope; CLI only, the `PackCreationService` pattern is reusable later).
- Migrating existing packs when `packs_dir` changes (v2 decision; this plan leaves existing packs where they are and documents it).

### Proposed scope

- **Minimal**: `config.toml` at `$XDG_CONFIG_HOME/rapm/config.toml` (fallback `~/.config/rapm/config.toml`; Windows `%APPDATA%\rapm\config.toml`) with `packs_dir` (writable root) and `extra_roots` (read-only). CLI: `rapm config set packs-dir <path>` / `rapm config get packs-dir` / `rapm config path`. Priority: `RAPM_PACKS_DIR` env > `packs_dir` > XDG default. TUI guard: close only when no readable pack anywhere; otherwise hint + read-only creation until configured.
- **Full** (same as minimal here; the two knobs are both small and the user asked for the location, plural roots are the natural shape of the same file).

### Decision requested

✅ Go on the minimal scope above (challenge points accepted as framed).

---

## Goal

Let the user choose, once and per-user, where the pack library lives (writable root + optional extra read-only roots) via a `config.toml` file and the `rapm config` CLI, with both the TUI and the CLI honoring it, and a startup guard when nothing is readable.

## Architecture decisions

- **D1: Config location & format** — `$XDG_CONFIG_HOME/rapm/config.toml`, else `$HOME/.config/rapm/config.toml`; on Windows `%APPDATA%\rapm\config.toml`. TOML via the existing `serde` + `toml` deps — **no new dependency**. Schema (all keys optional, unknown keys rejected):
  ```toml
  packs_dir = "/home/u/my-packs"      # writable root: where `pack create` writes
  extra_roots = ["/opt/team/packs"]   # read-only, appended after the system root
  ```
- **D2: Priority order** — `RAPM_PACKS_DIR` (env var, wins for scripts/CI, per XDG convention) > `packs_dir` (config) > XDG/`%APPDATA%` default. `extra_roots` are appended **after** the system root in read order, never before the writable root. Relative `packs_dir` is rejected (absolute required) — a library must not be cwd-dependent.
- **D3: Domain** — a `PackLibraryPath` newtype in `src/domain/pack/value_objects.rs`: `new(path)` validates absolute + non-empty, rejects `~` (no shell expansion in a config file). `DomainError::InvalidLibraryPath(String)` on failure. Pure, co-located UT.
- **D4: Application** — new bounded context `src/application/config/`:
  - `dto.rs`: `AppConfigDto { packs_dir: Option<PathBuf>, extra_roots: Vec<PathBuf> }`, `ResolvedConfigDto { packs_dir: PathBuf, extra_roots: Vec<PathBuf>, source: ConfigSource }` where `ConfigSource ∈ {Env, File, Default}` (lets the UI say *why* the path is what it is).
  - `traits.rs`: `#[automock] pub trait ConfigRepository: Send { fn load(&self) -> Result<AppConfigDto, ApplicationError>; fn save(&self, dto: &AppConfigDto) -> Result<(), ApplicationError>; }`.
  - `service.rs`: `ConfigService::resolve(env_value, config_dto) -> ResolvedConfigDto` (pure priority logic, UT) and `set_packs_dir(path)` / `get_packs_dir()` (validate via D3 → save). New `ApplicationError::InvalidConfig { path, message }` and reuse of `Io`.
- **D5: Infrastructure** — `src/infrastructure/config/fs_config.rs`: `FsConfigRepository` reads/writes the TOML at the XDG config path (`missing file` = empty config, not an error; malformed TOML = `InvalidConfig`; write is atomic: temp file + rename in the same dir). `src/infrastructure/pack/roots.rs` (new): the shared `is_pack_dir(dir)` helper (directory + `pack.toml` present) used by loader/content/explorer — **fixes review finding F1/F5 in the same pass**.
- **D6: Presentation — `PackLibrary` becomes config-aware** (`src/presentation/library.rs`): `PackLibrary::discover()` gains a config input — `discover_with(config: &ResolvedConfigDto)`. Read roots = `[packs_dir] ++ existing([exe/packs, cwd/packs, /usr/share/.../packs]) ++ extra_roots`; `user_root = packs_dir`. `discover()` (no config) keeps today's behavior for compatibility. The env-var read moves here from raw `std::env` to a parameter so UT stay pure. `ensure_user_root` stays presentation-side **only** as a call into a new tiny infra initializer, or is dropped in favor of `FsPackRepository::new` creating/tolerating the first root — **fixes F2**.
- **D7: Presentation — CLI `rapm config`** (`src/presentation/cli.rs`): subcommands `config set packs-dir <path>`, `config get packs-dir`, `config path` (prints the config file location), plus `rapm --version` (prints `env!("CARGO_PKG_VERSION")`, exit 0) — **fixes F6**. Parser extension is additive; exit codes unchanged (0/1/2). `config set` validates (absolute, parent creatable) and creates the directory.
- **D8: TUI startup guard** (`src/main.rs` + `src/presentation/tui/app.rs`): after library resolution, if `repository.list()` is empty **and** no pack is readable in any root → print a message on the terminal (before `ratatui::init()`, or a popup with quit as the only action if already in the TUI) naming `rapm config set packs-dir <path>`, and the only action is quit. If packs ARE readable (system packs via rpm/deb), the TUI opens normally and shows a non-blocking hint when `packs_dir` is unconfigured (creation disabled until set). No new screen in this iteration; the hint reuses the existing `Popup` mechanism (`app.rs:174`).
- **D9: Wiring (`main.rs`)** — resolve config (`ConfigService` + `FsConfigRepository`) → `PackLibrary::discover_with` → existing `build_app`/`build_creation_service`. `anyhow` stays in `main.rs` only.
- **D10: Tests** — per `rules/testing.md`:
  - UT: `PackLibraryPath` invariants; `ConfigService::resolve` priority matrix (env > file > default, extra_roots ordering); `FsConfigRepository` round-trip on tempdir (missing file, malformed TOML, atomic write); parser UT for `config`/`--version`.
  - IT CLI end-to-end (new `tests/config_cli.rs`): spawn `env!("CARGO_BIN_EXE_rapm")` with `HOME`/`XDG_CONFIG_HOME`/`XDG_DATA_HOME` on a tempdir — `config set` then `pack create` writes into the configured dir; `config get`; exit codes 0/1/2 — **fixes F4**.
  - IT TUI (extend `tests/split_screen_ui.rs` with `TestBackend`): library resolved from a tempdir config lists the configured packs (**fixes F3**); no readable pack → guard message + quit-only.
- **D11: Docs** — MANUAL.md (config file location, schema, priority, `rapm config`, startup guard behavior), README.md section, CONTEXT.md status line, this plan archived as implemented. Packaging unchanged (no file shipped in rpm/deb; the config is created on demand per user).

## Steps

1. **Domain** — `PackLibraryPath` + `DomainError::InvalidLibraryPath` + UT — `src/domain/pack/value_objects.rs`, `error.rs` — verify: `cargo test --lib domain`
2. **Application** — `src/application/config/` (DTOs, `ConfigRepository` port, `ConfigService` with priority logic) + `ApplicationError::InvalidConfig` + mockall UT — verify: `cargo test --lib application`
3. **Infrastructure** — `FsConfigRepository` (tempdir UT: round-trip, missing, malformed, atomic) + `src/infrastructure/pack/roots.rs` shared `is_pack_dir` adopted by loader/content/explorer (fixes F1/F5) + UT — verify: `cargo test --lib infrastructure`
4. **Presentation — library** — `PackLibrary::discover_with(config)`, env injected as parameter, `ensure_user_root` moved/tolerated in infrastructure (fixes F2) + UT on the priority matrix — verify: `cargo test --lib presentation`
5. **Presentation — CLI** — `rapm config set|get|path`, `rapm --version` (fixes F6) + parser UT — verify: `cargo test --lib presentation`
6. **Wiring + TUI guard** — `main.rs` config resolution, startup guard (quit-only when nothing readable, hint otherwise) in `app.rs` — verify: `cargo build`
7. **IT** — `tests/config_cli.rs` (binary spawn, fixes F4) + `tests/split_screen_ui.rs` scenario (fixes F3) — verify: `cargo test --test config_cli --test split_screen_ui`
8. **Docs** — MANUAL.md, README.md, CONTEXT.md, archive plan — verify: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`

## Risks and mitigations

- **Existing users' packs become invisible** after switching `packs_dir` → document in MANUAL.md; v2 may offer `rapm config migrate`. Accepted for v1 of this feature.
- **Config file with a bad path breaks the TUI** → invalid TOML or non-absolute `packs_dir` = `InvalidConfig` error with the file path, TUI falls back to defaults and shows the error as a popup (never a panic; `rules/presentation.md` §5).
- **Env var vs config confusion** → `rapm config get packs-dir` prints the value **and its source** (`env` / `config` / `default`).
- **Windows path handling** → `PackLibraryPath` uses `PathBuf` semantics; `%APPDATA%` branch mirrors the existing `user_packs_dir_from` pattern; CI already builds Windows.

## Out of scope

- rpm/deb install-time prompt for the packs path (rejected: per-user setting, packages stay system-wide read-only).
- A TUI screen to edit the config (CLI only in this iteration).
- Automatic migration of packs when `packs_dir` changes (v2 decision).
- `rapm config unset` / editing `extra_roots` from the CLI (v2; the file is documented for manual editing).
- Fixing the symlink-cycle recursion in `copy_dir` (review F7 — separate bugfix, regression test first per `rust-bugfix`).
