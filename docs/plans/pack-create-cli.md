# Plan: user pack library (XDG) + `rapm pack create` CLI

> Status: **implemented** — user pack library (XDG) + `rapm pack create` CLI. Source: skills `/challenge` then `/plan`. User decisions are marked ✅.

## Goal

Let the user build a pack from an arbitrary list of files/directories (skills, agents, commands, any layout) and register it in the pack library where the tool is installed, so the TUI sees it live. Delivered as a **CLI subcommand** of the existing `rapm` binary: `rapm pack create ...` generates the pack directory + `pack.toml` (canonical `<kind>s/<name>` layout) and writes it into a new **user pack library** that takes precedence over the system one.

## Framing context (challenge)

- Restatement: given a list of files/directories, generate a pack (directory + `pack.toml` manifest) and copy it into the pack library so the running/later TUI can install it. Two options were analyzed: a CLI subcommand vs new TUI screens (home screen + action list + creation wizard).
- Why CLI-first: the expensive, reusable core (classify items, compute per-CLI destinations, copy files, serialize the manifest) is identical in both options; the TUI variant additionally requires multi-file selection in the explorer (does not exist today) plus keyboard forms for name/description/destinations — roughly 2-2.5x the work. The TUI already reloads the library on every action (`refresh_installed`, `src/presentation/tui/app.rs:151`), so a pack created externally appears without any TUI change. A future "import as pack" TUI screen can reuse the same `PackCreationService`.
- User decisions:
  - ✅ **CLI subcommand of the existing binary** (`rapm pack create`), no second binary, no packaging change (RPM/DEB/7z ship `rapm` only).
  - ✅ **User pack library** at the XDG data dir (`$XDG_DATA_HOME/rapm/packs`, else `$HOME/.local/share/rapm/packs`; Windows: `%APPDATA%\rapm\packs`), **first in the read order**, ahead of `./packs` next to the binary, `./packs` in cwd, `/usr/share/rusty-agentic-pack-manager/packs`. Creation always writes there (created on demand). A user pack overrides a system pack of the same name.
  - ✅ **Explicit item options**: `--skill|--agent|--command <path>` (repeatable; file or directory) followed by `--vibe-dest <d>` / `--gemini-dest <d>` for that item. No kind auto-detection.
  - ✅ **Metadata**: `--name` (prompted interactively if absent and a TTY is available, hard error otherwise), `--version` (default `1.0.0`), `--description` (prompted if absent and TTY, empty otherwise), `--cli vibe,gemini` (default: union of the items' destination CLIs), `--force` (replace an existing pack).
  - ✅ **Canonical layout inside the pack**: each item is copied under `<kind>s/<file-or-dir-name>/` (e.g. `skills/tdd/`, `commands/review.toml`).
- Accepted limits (documented in MANUAL.md): the TUI shows a pack created while it runs only after the next action (no dedicated refresh key in this iteration); overwrite is best-effort (old pack moved aside, then removed — a crash between the two renames can leave a `<name>.old-<pid>` directory).

## Architecture decisions

- **D1: Domain additions** — `PackItemKind::plural_dir()` (`skills`/`agents`/`commands`) and `DomainError::DuplicateItemSource(String)` (two sources of the same kind with the same file/dir name would collide in the canonical layout). Pure layer, co-located UT.
- **D2: Application port `PackWriter`** (`src/application/pack_create/traits.rs`, `#[automock]`, `Send`):
  ```rust
  fn pack_exists(&self, name: &str) -> bool;
  fn write(&self, pack: &Pack, sources: &[PackSourceDto]) -> Result<PackWriteOutcomeDto, ApplicationError>;
  ```
  `PackSourceDto { host_path: PathBuf, pack_source: PathBuf }` (absolute-or-cwd-relative host path → canonical path inside the pack). `PackWriteOutcomeDto { pack_dir: PathBuf, copied_count: usize }`. New `ApplicationError` variants: `PackAlreadyExists(String)`, `Usage(String)` (CLI usage errors; `anyhow` stays confined to `main.rs`).
- **D3: `PackCreationService`** (`src/application/pack_create/service.rs`, stateless, `Box<dyn PackRepository>` + `Box<dyn PackWriter>`) — `create(dto: CreatePackDto) -> Result<CreatePackResultDto, ApplicationError>`. Orchestration: parse/validate `PackName`, `PackVersion`, kinds (`PackItemKind::parse`) and CLI names (`TargetCli::parse`); deduce `clis` from item destinations when `dto.clis` is empty; compute each item's canonical source (`<plural_dir>/<file_name>`); reject duplicate canonical sources (D1); validate that **every item has a destination for every declared CLI** (reuses `DomainError::EmptyItemDestination` via `#[from]`); build the `Pack`; reject an existing pack unless `overwrite` (`PackAlreadyExists`); delegate the write to the port. No partial pack on validation errors: all checks run before `write`.
- **D4: `FsPackWriter`** (`src/infrastructure/pack/writer.rs`) — writes **only** into the user library root. Phase 1 validates every source (exists; file or dir). Phase 2 stages: copy each source to `<user_root>/.staging-<name>-<pid>/<canonical>` (recursive for directories, reusing the `copy_dir` semantics — existing destinations never silently overwritten inside a fresh staging dir), serialize `pack.toml` into the staging dir, then swap: rename existing `<name>` to `<name>.old-<pid>` (only with `overwrite`), rename staging → `<name>`, remove the `.old` backup. Any failure cleans the staging dir. The manifest struct in `loader.rs` becomes `pub(crate)` + `Serialize` and is shared by reader and writer (single source of truth for the format).
- **D5: Multi-root library** — `FsPackRepository::new(roots: Vec<PathBuf>)`, `FsInstallDestination::new(roots: Vec<PathBuf>)`, `FsPackContentRepository::new(roots: Vec<PathBuf>)` (signature change from a single `PathBuf`; existing UT/IT call sites updated). `list()` = union over roots, deduplicated by pack name (first root wins), sorted; `find(name)` / `pack_dir(name)` / install-source resolution = first root containing the pack. Read order: user library → `./packs` next to the binary → `./packs` in cwd → `/usr/share/rusty-agentic-pack-manager/packs`.
- **D6: Presentation `PackLibrary`** (`src/presentation/library.rs`) — testable discovery: `PackLibrary { roots: Vec<PathBuf>, user_root: PathBuf }` with `PackLibrary::discover()` (env/home/exe/cwd/system resolution, pure core `resolve_roots(...)` injected with the env values for UT) and `PackLibrary::new(roots, user_root)` for tests/IT. `main.rs`'s `find_packs_dir()` is replaced by it.
- **D7: Presentation CLI** (`src/presentation/cli.rs`) — `dispatch(args: &[String], lib: &PackLibrary) -> Result<Option<i32>, ApplicationError>`: `None` = not a CLI invocation (launch the TUI); `Some(code)` = CLI ran, `main` exits with it. Subcommands: `pack create` (parser `parse_create_args` — pure, UT), `--help`/`help` (usage, exit 0), unknown args → `Usage` (exit 2). Item flags accumulate into the current item (`--vibe-dest`/`--gemini-dest` attach to the last `--skill|--agent|--command`); missing destination for a declared CLI is caught by the service (D3). Interactive prompts read a line from stdin only when `std::io::IsTerminal` is true; otherwise `--name` missing = `Usage` error, `--description` missing = empty. Success prints the created pack path + item count.
- **D8: Wiring (`main.rs`)** — dispatch the CLI **before** `ratatui::init()` (the CLI never touches raw mode); `anyhow` stays in `main.rs` only. The TUI path is unchanged except for the multi-root constructors (D5).
- **D9: TUI sees new packs for free** — `refresh_installed()` (`app.rs:151`) re-lists the library on every action; no new key, no new screen in this iteration.
- **D10: Docs** — MANUAL.md (CLI usage, user library location + precedence, canonical layout, accepted limits), README.md, CONTEXT.md updated; this plan archived.

## Steps

1. Domain — `plural_dir`, `DuplicateItemSource` + UT — `cargo test --lib domain`
2. Application — `pack_create/` (DTOs, `PackWriter`, `PackCreationService`), `ApplicationError::{PackAlreadyExists, Usage}` + UT mockall — `cargo test --lib application`
3. Infrastructure — loader/content/explorer take `Vec<PathBuf>` roots (union, dedup, first-hit) + UT updated — `cargo test --lib infrastructure`
4. Infrastructure — `FsPackWriter` (validate → stage → swap) + UT on tempdir — `cargo test --lib infrastructure`
5. Presentation — `library.rs` (`PackLibrary`, XDG resolution) + UT — `cargo check`
6. Presentation — `cli.rs` (arg parser, prompts, dispatch) + UT parser — `cargo test --lib presentation`
7. Wiring — `main.rs` CLI dispatch before `ratatui::init()`, multi-root constructors — `cargo check`
8. IT — `tests/pack_create.rs` (end-to-end create → loadable via `FsPackRepository`, manifest content, overwrite, duplicate-source rejection, missing-destination rejection, user-root precedence over a system root) + adapt `tests/pack_install.rs` / `tests/split_screen_ui.rs` to the new constructors — `cargo test`
9. Docs — MANUAL.md, README.md, CONTEXT.md, this plan — `cargo fmt --check`
10. Quality gate — `cargo fmt` → `cargo clippy --all-targets -- -D warnings` → `cargo test` → `cargo check`

## Risks and mitigations

- **Cross-device rename** — staging lives inside the user library root (same filesystem as the final rename).
- **Partial creation** — validation of all sources happens before any write; staging + swap keeps the final pack complete or absent; staging cleaned on error.
- **Overwrite crash window** — old pack moved to `<name>.old-<pid>` before the swap and removed after; a crash can leave the `.old` directory (documented accepted limit, no silent data loss).
- **No TTY in CI** — missing `--name` without a TTY is an explicit `Usage` error, not a hang.
- **User pack shadows a system pack** — by design (first root wins); documented in MANUAL.md.
- **Stale TUI list** — a pack created while the TUI runs appears after the next action; documented (no refresh key in this iteration).
- **Env-dependent discovery in tests** — `resolve_roots` takes env values as parameters; tests never touch the real environment (rules/testing.md).

## Out of scope

- TUI creation screen / home screen with action list (the `PackCreationService` is built for it later).
- Custom library paths via a configuration file.
- Other subcommands (`pack list`, `pack remove`).
- Dedicated TUI refresh key.
- Transactional rollback if the `.old` backup removal fails (best effort, documented).
- 7z/tar packs, CLIs other than vibe/gemini (unchanged).
