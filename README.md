# Rusty Agentic Pack Manager

A Rust TUI to manage agentic packs: creation, deployment and removal of skill, agent and command sets in any repository.

## What is an agentic pack?

A **pack** is a coherent combination of skills, agents and commands, applicable to a given CLI (Vibe CLI, Gemini CLI, …) and easily deployable — and removable — in a repository.

- **Format**: a directory containing a `pack.toml` manifest (name, description, version, target CLIs, items) and the pack files (skills, agents, commands).
- **Granularity**: thematic packs (e.g. `tdd`, `review`) grouping related skills + agents + commands, plus solo packs.
- **Installation**: the manifest declares, per CLI, the destinations (e.g. `.agents/skills/` for Vibe, `.gemini/commands/` for Gemini); files are copied under the directory chosen by the user.

## UI — split screen

- **Left panel**: the pack list, grouped by target CLI (Vibe CLI, Gemini CLI, …).
- **Right panel**: the file tree of the host machine.

| Key | Action |
|---|---|
| `Tab` | Switch focus between panels |
| `Enter` / `Backspace` | Navigate (down / up) in the list or the file system |
| `A` | Install the selected pack in the explorer's current directory |
| `V` | View the pack files (opens on `pack.toml`) with syntax highlighting (packs panel only) |
| `R` | Create a folder (explorer) |
| `M` | Rename a folder/file (explorer) |
| `D` | Delete a folder/file (explorer; `Shift-D` forces non-empty folders) |
| `Space` | Contextual help (depending on the active panel / mode) |

## Architecture

Rust (edition 2021, MSRV 1.88), `ratatui` + `crossterm` TUI. DDD 4 layers, strictly inward flow:

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Architecture rules, design and action patterns are taken from [rusty-console](https://github.com/TataneCode/rusty-console).

- Agents entry point: [AGENTS.md](AGENTS.md), then [CONTEXT.md](CONTEXT.md)
- Per-layer rules: [rules/](rules/)
- Design plans: [docs/plans/](docs/plans/)

## Embedded packs

The initial pack list (in `packs/`) is built from [docs-tech/llm-cli](https://github.com/TataneCode/docs-tech/tree/main/llm-cli) and rusty-console, made library-agnostic, targeting **Vibe CLI** and **Gemini CLI** first.

## Packaging

- **RPM / DEB**: the `packs/` directory ships inside the package (e.g. `/usr/share/rusty-agentic-pack-manager/packs/`).
- **Windows**: a 7z archive is generated as an artifact (binary + a `packs` directory side by side).

## Status

V1 under construction — see [docs/plans/init.md](docs/plans/init.md).
