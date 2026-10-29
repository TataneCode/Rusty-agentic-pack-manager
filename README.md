# Rusty Agentic Pack Manager

A Rust TUI to manage agentic packs: creation, deployment and removal of skill, agent and command sets in any repository.

## What is an agentic pack?

A **pack** is a coherent combination of skills, agents and commands, applicable to a given CLI (Vibe CLI, Gemini CLI, …) and easily deployable — and removable — in a repository.

- **Format**: a directory containing a `pack.toml` manifest (name, description, version, target CLIs, items) and the pack files (skills, agents, commands).
- **Granularity**: thematic packs (e.g. `tdd`, `review`) grouping related skills + agents + commands, plus solo packs.
- **Installation**: the manifest declares, per CLI, the destinations (e.g. `.agents/skills/` for Vibe, `.gemini/commands/` for Gemini); files are copied under the directory chosen by the user.

## UI — split screen

- **Left column**: the available pack list, grouped by target CLI (Vibe CLI, Gemini CLI, …), and below it the **Installed** panel: packs already installed in the explorer's current directory (tracked in `packs_index.toml`).
- **Right panel**: the file tree of the host machine.

| Key | Action |
|---|---|
| `Tab` | Switch focus (Packs → Installed → Explorer) |
| `Enter` / `Backspace` | Navigate (down / up) in the list or the file system |
| `A` | Install the selected pack in the explorer's current directory |
| `V` | View the pack files (opens on `pack.toml`) with syntax highlighting (packs panel only) |
| `U` | Uninstall the selected pack: remove every file it installed (installed panel) |
| `R` | Create a folder (explorer) |
| `M` | Rename a folder/file (explorer) |
| `D` | Delete a folder/file (explorer; `Shift-D` forces non-empty folders) |
| `Space` | Contextual help (depending on the active panel / mode) |

## Installation index

Each install is recorded in a `packs_index.toml` file at the root of the target directory — one entry per `(pack, CLI)` with the relative paths actually copied. The **Installed** panel reads this index: it shows what is installed in the current directory, hides those packs from the available list, and `U` uninstalls a pack by removing exactly the recorded assets.

## Architecture

Rust (edition 2021, MSRV 1.88), `ratatui` + `crossterm` TUI. DDD 4 layers, strictly inward flow:

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Architecture rules, design and action patterns are taken from [rusty-console](https://github.com/TataneCode/rusty-console).

- Agents entry point: [AGENTS.md](AGENTS.md), then [CONTEXT.md](CONTEXT.md)
- Per-layer rules: [rules/](rules/)
- Design plans: [docs/plans/](docs/plans/)

## Pack library and pack creation

Packs are searched, in order: the **user library** (`$XDG_DATA_HOME/rapm/packs/` or `~/.local/share/rapm/packs/`, `%APPDATA%\rapm\packs` on Windows), `<binary directory>/packs/`, `./packs`, then `/usr/share/rusty-agentic-pack-manager/packs/` (system install). The first location containing a pack wins: a user pack overrides a system pack of the same name.

New packs are created from any list of files/directories with the CLI:

```bash
rapm pack create --name my-pack \
  --skill ./skills/tdd --vibe-dest .agents/skills/tdd/ --gemini-dest .gemini/skills/tdd/ \
  --command ./commands/review.toml --vibe-dest .vibe/commands/review.toml --gemini-dest .gemini/commands/review.toml
```

The pack is written into the user library (canonical layout `<kind>s/<name>/` + a generated `pack.toml`) and is immediately available in the TUI. See [MANUAL.md](MANUAL.md) for the full option list.

## Embedded packs

The initial pack list (in `packs/`) is built from [docs-tech/llm-cli](https://github.com/TataneCode/docs-tech/tree/main/llm-cli) and rusty-console, made library-agnostic, targeting **Vibe CLI** and **Gemini CLI** first.

## Packaging

- **RPM / DEB**: the `packs/` directory ships inside the package (e.g. `/usr/share/rusty-agentic-pack-manager/packs/`).
- **Windows**: a 7z archive is generated as an artifact (binary + a `packs` directory side by side).

## Status

V1 under construction — see [docs/plans/init.md](docs/plans/init.md).
