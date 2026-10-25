# Plan: rusty-agentic-pack-manager — v1 foundation (init)

> Status: **implemented** — v1 complete on branch vibe/v1-implementation. Source: skills `/challenge` then `/plan`. User decisions are marked ✅.

## Goal

Deliver a Rust TUI (split screen) to browse a list of embedded agentic packs (sorted by CLI), navigate the host filesystem, and install a pack's content in the chosen directory — reusing the DDD 4-layer architecture, design (theme, widgets, presenter/view/actions triad, event loop) and action patterns from rusty-console.

## Framing context (challenge)

- Restatement: TUI managing the lifecycle of agentic packs (skills/agents/commands per target CLI), deployable and removable in any repository.
- User decisions:
  - ✅ **Pack = directory + `pack.toml`** (7z/tar archives in v2).
  - ✅ **Thematic granularity (option B)**: packs grouping related skills + agents + commands (e.g. `tdd`), plus solo packs; about 6-8 packs + solos.
  - ✅ **Apply flow**: `A` on a pack → focus on the explorer panel in install mode; only available actions: navigation (Enter/Backspace), `I` = install in the selected directory, Space = contextual help.
- Scope creep dropped from v1: rename (`M`), delete (`D`/`Shift-D`) on the host FS — **later re-included by user decision**; full undeploy (mitigation: copy the manifest into the target at install to make it possible in v2).

## Architecture decisions

- **D1: Synchronous application, no tokio/bollard** — the v1 scope is purely local (FS + TUI); tokio, bollard, async-trait, futures-util, chrono, base64 are not carried over. `std::fs` + the crossterm event loop suffice (rusty-console `EventHandler` pattern kept, without async).
- **D2: Pack = directory + `pack.toml`** ✅ — `toml` + `serde` dependencies. Manifest schema: `name`, `description`, `version`, `cli` (target list: `vibe`/`gemini`), `[[items]]` (kind `skill|agent|command`, local source, per-CLI destination: e.g. `.agents/skills/<name>/` for vibe, `.gemini/commands/` for gemini).
- **D3: Pack discovery on disk** — search in `<binary dir>/packs/` then `/usr/share/rusty-agentic-pack-manager/packs/`. Makes packaging trivial (RPM/DEB embed `packs/` as an asset; the Windows 7z = binary + `packs/` side by side).
- **D4: Domain `pack`** — entity `Pack` (identity = name + version), value objects `PackName`, `PackVersion`, `TargetCli` (extensible enum), `PackItemKind`; dedicated errors in `01_domain/error.rs`. **Install planning** (source→destination copy computation, collision detection) is a domain rule, testable without the FS.
- **D5: Two application areas** — `pack` (list/load service, `PackRepository` trait) and `explorer` (FS navigation service, `ExplorerRepository` traits: list, create_dir; `FileNode` DTO). Adapters in `03_infrastructure/filesystem/` (std::fs) and `03_infrastructure/pack/` (`pack.toml` reading). Boundaries mocked with `mockall` in tests, like rusty-console.
- **D6: UI = one screen with two panels** — state `Focus::Packs | Focus::Explorer` toggled by Tab. The "apply" flow introduces an `Installing { pack }` mode forcing `Focus::Explorer` and restricting actions to: navigation, `I` = install, Space = help. Presenter/view/actions triad per panel + `common/` from rusty-console (`theme`, `keys`, `action_bar`, contextual help popup, `confirm`). `App`/`router.rs` sequence the `AppAction`, pattern identical to rusty-console.
- **D7: v1 keys** — Tab (focus), Enter/Backspace (navigation on both sides), `A` (apply → install mode), `I` (install here), `R` (create folder), `V` (pack viewer, packs panel only), `M` (rename, explorer), `D`/`Shift-D` (delete, explorer — user decision added post-challenge), Space (contextual help per panel/mode).
- **D8: Initial pack content — thematic granularity** ✅, vibe + gemini only, library-agnostic. Packs shipped: `tdd`, `code-review`, `debugging`, `security`, `planning`, `brainstorming`, `quality-gate`, plus solo packs (`prd`, `work-slicing`, `karpathy`, `parallel-agents`, `focus-adhd`) from docs-tech/llm-cli; `rusty-agnostic` pack from rusty-console skills/agents/commands.
- **D9: Pack viewer (`V`, packs panel only)** ✅ — `V` on a selected pack opens a file-viewing screen: `.md`, `.js`, `.toml` (any text file in the broad sense; non-text files are cleanly refused). The viewer reuses the rusty-console "logs screen" pattern: full screen, navigation (arrows, PgUp/PgDn, Home/End), `Esc` returns to the split screen, `Tab` switches between the pack file list and the content preview. **Syntax highlighting** via `syntect` (`.md`, `.js`, `.toml` syntax readers) ; rendering adapted to the terminal 16 colors via the rusty-console theme.
- **D10: Rename and delete in the explorer (v1)** ✅ (user decision) — `M` renames the selected file/folder via the input dialog prefilled with the current name; `D` deletes with a confirmation dialog; `Shift-D` forces the deletion of a non-empty folder. The application service validates names (no path separators, no `..`); deletion is recursive and confined to the explorer's current directory.

## Steps

1. **Skeleton** — `Cargo.toml` (deps: ratatui 0.30, crossterm 0.29, thiserror, anyhow, serde, toml; dev: mockall), `src/lib.rs`, `src/main.rs`, empty 4-layer modules — *check: `cargo check`*
2. **Domain pack** — `01_domain/pack/{entity,value_objects}.rs`, `01_domain/error.rs` + unit tests (manifest validation, install plan, collisions) — *check: `cargo test --lib pack`*
3. **Application** — `02_application/{pack,pack_view,explorer}/` (traits, DTOs, services, mappers) + tests with `mockall` mocks — *check: `cargo test --lib`*
4. **Infrastructure** — `03_infrastructure/pack/` (`pack.toml` loader) and `03_infrastructure/filesystem/` (explorer, create_dir) + tempdir tests — *check: `cargo test --lib`*
5. **TUI foundation** — `04_presentation/tui/{app,event,router}.rs` + `common/` (theme, keys, action_bar) from rusty-console; split screen with two panels, Tab toggles focus, empty render — *check: `cargo test --lib presentation`*
6. **Packs panel** — triad `packs/{presenter,view,actions}.rs`: CLI-grouped list, selection, `A` enters install mode — TestBackend tests — *check: `cargo test packs`*
7. **Explorer panel** — triad `explorer/{presenter,view,actions}.rs`: Enter/Backspace navigation, `R` (create folder, input dialog), restricted install mode + `I` installs (confirm dialog) — TestBackend tests — *check: `cargo test explorer`*
8. **Contextual help** — Space popup per focus/mode — TestBackend test — *check: `cargo test help`*
9. **Pack viewer (`V`)** — application `pack_view` (trait + DTO + service + mockall tests); infrastructure `03_infrastructure/pack/content.rs`; presentation `04_presentation/tui/viewer/` (triad: file list ↔ highlighted preview, syntect, logs-like navigation) + TestBackend tests — *check: `cargo test viewer`*
10. **Integration tests** — `tests/pack_install.rs` (real pack → temp dir: files copied to the right destinations), `tests/split_screen_ui.rs` (Tab focus, panel rendering) — *check: `cargo test`*
11. **Data: pack writing** — `packs/<name>/pack.toml` + content from docs-tech/llm-cli and rusty-console, vibe/gemini-adapted, agnostic — *check: `cargo test --test pack_install` + manual manifest review*
12. **Packaging** — `cargo-generate-rpm` (assets: binary + `packs/`), `cargo-deb`, Windows 7z artifact (binary + `packs/`) in CI — *check: `cargo generate-rpm` local*
13. **Docs** — `README.md` (done), `MANUAL.md` (keys), `CONTEXT.md`/`AGENTS.md` adapted to this project, plan archived in `docs/plans/` (this file) — *check: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`*
14. **Rename / delete (v1 addition)** — application: `rename`, `remove` (recursive, confined) on `ExplorerRepository`; infrastructure: `FsExplorer` implementations; UI: `M` (prefilled input dialog), `D` (confirm dialog), `Shift-D` (force non-empty folder); docs and help updated; TestBackend + tempdir tests — *check: `cargo test`*

## Risks and mitigations

- **Exact gemini install path** (`.gemini/commands/` to confirm) → per-pack manifest = source of truth editable without recompiling; check at step 11.
- **rusty-console patterns to de-asyncify** (tokio removed) → mitigated by keeping the synchronous event loop simple from step 5.
- **`toml` crate vs MSRV 1.88** → check the resolved version at step 1, pin if needed.
- **7z on Windows in CI** → requires `7z` on the runner; fallback: zip archive if the tool is missing.
- **`syntect` vs MSRV 1.88 / binary size** → syntect pulls embedded syntax dumps (~2-3 MB); if MSRV or size becomes a problem, fallback: homegrown lexical highlighting (std regex) limited to `.md`/`.js`/`.toml`, decided at step 9.
- **16-color terminal mapping** → syntect outputs 24-bit colors; map to the theme palette; TestBackend test on styled spans.
- **Destructive delete on the host FS** → confirmation dialog, recursive delete confined under the explorer's current directory, no `..` or absolute paths accepted by the service.

## Out of scope (v1)

- Full undeploy of an applied pack — v2 (the manifest copied into the target at install will make it possible)
- 7z/tar pack formats — v2
- CLIs other than vibe and gemini — structure ready (`TargetCli` enum), content not provided

## Translations (v1 addition)

All project resources are translated to English, pack skills included: `packs/` (manifests stay language-neutral TOML; SKILL.md/commands/agents content), `rules/`, `agents/`, `.vibe/` (prompts, agent descriptions, config), `README.md`, `MANUAL.md`, `AGENTS.md`, `AGENT_ARCHITECTURE.md`, `CONTEXT.md`, this plan.
