# rusty-agentic-pack-manager — Project Map (CONTEXT.md)

**Single entry point.** Read this file first; it tells you what to read next — and what NOT to read. Golden rule: before browsing the code, check the map below and limit your reading to the task's scope.

## In 30 seconds

Rust TUI to manage **agentic packs**: sets of skills, agents and commands applicable to a target CLI (Vibe CLI, Gemini CLI) and installable in any repository. Stack: `ratatui` + `crossterm` (TUI), `syntect` (highlighting), `serde` + `toml` (manifests). Edition 2021, **MSRV 1.88**. No daemon, no network: pure local filesystem. The design, action patterns and agentic tooling come from [rusty-console](https://github.com/TataneCode/rusty-console).

## Architecture — DDD 4 layers, strictly inward flow

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

No outer layer may import an inner layer. The UI never sees the domain: DTOs + service traits only.

## Navigation map — what to read, when

| You want to… | Read | Do NOT read |
|---|---|---|
| Understand the project | This file + [README.md](README.md) | All of AGENTS.md, all of src/ |
| Keys / usage | [MANUAL.md](MANUAL.md) | — |
| Frame a request | `/challenge` skill | — |
| Plan a feature | `/plan` + `rules/README.md` | All of src/ before framing |
| Pack format / manifest | `packs/tdd/pack.toml` + [MANUAL.md](MANUAL.md) | — |
| Code — domain layer | `rules/domain.md` + `src/01_domain/pack/` | The other layers |
| Code — application layer | `rules/application.md` + `src/02_application/{pack,pack_view,explorer}/traits.rs` | `03_infrastructure` |
| Code — infrastructure layer | `rules/infrastructure.md` + `src/03_infrastructure/` | `04_presentation` |
| Code — UI | `rules/presentation.md` + `src/04_presentation/tui/` | `01_domain` directly |
| Write tests | `rules/testing.md` + `rust-testing` skill | — |
| Delivery / quality | `rules/quality.md` (fmt, clippy, tests) | — |
| Review a diff / a PR | `/self-review` then `/review` + `rules/` | — |
| Plan history | `docs/plans/` | — |

## Project status

**V1 in progress** — delivered foundation: packs/explorer split screen, apply (`A`) + install (`I`), viewer (`V`) with syntax highlighting, folder creation (`R`), rename (`M`), delete (`D`/`Shift-D`), contextual help (Space), 16 embedded packs, RPM/DEB/7z packaging in CI. Plan: [docs/plans/init.md](docs/plans/init.md).

Out of scope for v1 (planned v2): undeploy, 7z/tar packs, CLIs other than vibe/gemini.
