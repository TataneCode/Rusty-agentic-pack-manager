# AGENTS.md — rusty-agentic-pack-manager

Startup router for any AI agent. **Do not read the whole project**: start with [CONTEXT.md](CONTEXT.md) (project map), check this table, then read only the files in the task's scope.

## Confirmation Gate (mandatory)

Before **any** write action on this repository (file, branch, commit, push, PR, merge), the agent MUST present a concise plan and wait for the user's explicit agreement. Purely read-only operations (read, grep, `cargo test`/`clippy`/`fmt --check`/`build`) are exempt. Silence or an ambiguous answer means "do not continue". Full procedure: [.agents/skills/confirmation-gate/SKILL.md](.agents/skills/confirmation-gate/SKILL.md).

## In 15 seconds

Rust TUI to manage **agentic packs** (skills, agents, commands) for AI CLIs (Vibe CLI, Gemini CLI): list, view, install in a repository. DDD 4 layers, strictly inward flow:

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Stack: ratatui + crossterm, syntect (highlighting), serde + toml (manifests), thiserror. **Synchronous** application (std::fs, no tokio). Edition 2021, MSRV 1.88. Commands and project status: [CONTEXT.md](CONTEXT.md).

## Routing — which command for which task

| Command (skill `/…`) | Role | When to use |
|---|---|---|
| `/challenge` | Challenger | **Before any `/plan`**: restate the request, detect ambiguities and scope creep, propose the minimal scope |
| `/plan` | Architect | Frame a feature, bugfix or refactoring before writing code (approved plan mandatory before implementation) |
| `/implement` | Developer | Implement from an approved plan (never commit/push without agreement) |
| `/self-review` | Developer | Reread your own diff (rules checklist, cross-layer imports, local gate) **before** requesting a review |
| `/review` | Reviewer | GitHub PR review (Vibe Code) or branch diff review (Vibe CLI), structured verdict |
| `/audit` | Auditor | Analyze a zone of the produced code: rules compliance, debt, DDD violations — read-only |
| `/update-config` | — | Evolve this configuration (skills, rules, agents, commands) |

Business skills invoked by the roles above: `rust-architecture-planning`, `rust-feature-ddd`, `rust-bugfix`, `rust-refactoring`, `rust-testing`, `rust-quality-gate`, `rust-dependency`, `rust-code-review`, `layer-rules` — see [.agents/skills/](.agents/skills/).

## Sources of truth (one per topic)

| Topic | File |
|---|---|
| Project context, navigation map | [CONTEXT.md](CONTEXT.md) |
| Per-layer DDD rules | [rules/](rules/) (`domain`, `application`, `infrastructure`, `presentation`) |
| Cross-cutting quality (commands, style) | [rules/quality.md](rules/quality.md) |
| UT & IT conventions | [rules/testing.md](rules/testing.md) |
| Pack format / manifest | [MANUAL.md](MANUAL.md) + `packs/tdd/pack.toml` (example) |
| Embedded packs | [packs/](packs/) |
| Agent/skill procedures | [.agents/skills/](.agents/skills/) + [AGENT_ARCHITECTURE.md](AGENT_ARCHITECTURE.md) |
| Agent roles (agnostic Markdown) | [agents/](agents/) |
| Vibe CLI wiring | [.vibe/](.vibe/) (`agents/`, `prompts/`, `config.toml`) |

## Non-negotiable conventions

- **Read a file entirely before modifying it**; minimal diff; match the existing style.
- **Errors**: `thiserror` per layer — `DomainError` → `ApplicationError` (`#[from]`) for the UI; `anyhow` reserved for `main.rs`. No error `String`s in the layers.
- **Tests**: every new logic has its test; bugfix = failing regression test first, then fix. See [rules/testing.md](rules/testing.md).
- **Gate before delivery**: `cargo fmt` → `cargo clippy -- -D warnings` → `cargo test`. See [rules/quality.md](rules/quality.md).
- **Never commit, push, PR or merge without the user's explicit agreement** (see Confirmation Gate above).

## Additional resources

- [README.md](README.md) — user docs; [MANUAL.md](MANUAL.md) — keys manual and pack format.
- [docs/plans/](docs/plans/) — per-feature plans.
- Design and patterns taken from [rusty-console](https://github.com/TataneCode/rusty-console) (see [AGENT_ARCHITECTURE.md](AGENT_ARCHITECTURE.md)).
