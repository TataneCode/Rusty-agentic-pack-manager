# User manual — rapm

`rapm` is the TUI for managing agentic packs: list, view, install and uninstall sets of skills, agents, rules and commands in a repository, for a target CLI (Vibe CLI, Gemini CLI, …). It also ships a minimal CLI with the fundamental commands only: install/uninstall a pack of the library, and add/remove a pack in the library.

## Startup

```bash
rapm
```

The application starts in the current directory. Packs are searched in:

1. **User library**: the `packs_dir` of the [configuration file](#configuration-file), else `$XDG_DATA_HOME/rapm/packs/` (or `~/.local/share/rapm/packs/`; `%APPDATA%\rapm\packs` on Windows) — writable, this is where `rapm add` writes
2. `<binary directory>/packs/`
3. `./packs` (current directory)
4. `/usr/share/rusty-agentic-pack-manager/packs/` (RPM/DEB system install)

The first location containing a pack wins: a pack in the user library overrides a system pack of the same name. Packs created while the TUI is running appear after the next action (the list is reloaded on every action).

## Configuration file

Both the CLI and the TUI read one TOML file, edited by hand:

- **Location**: `$XDG_CONFIG_HOME/rapm/config.toml`, else `~/.config/rapm/config.toml` (`%APPDATA%\rapm\config.toml` on Windows).
- **Schema** (every key optional):

  ```toml
  packs_dir = "/home/u/my-pack-library"  # writable pack library (absolute path required)
  default_cli = "vibe"                    # default target CLI of installs
  ```

- A missing file means an empty config (no error); a malformed file or a relative `packs_dir` is an error naming the file.
- `packs_dir` replaces the user library (first read root and writable root, where `rapm add` writes); the other search locations stay. `default_cli` must name a loaded CLI profile (`clis/*.toml`); when a pack does not declare it, the pack's first declared CLI is used.

## Adding a pack to the library: `rapm add <dir>`

```bash
rapm add <dir> [--name <name>] [--force]
```

The base directory **may** contain `agents/`, `skills/`, `rules/` and `commands/` — each optional, at least one required. Each entry becomes one pack item, copied under the canonical layout `<kind>s/<name>/` (directories are copied recursively, e.g. `skills/tdd/`); anything else at the base root is ignored. The `pack.toml` is generated and traces the whole content: name (`--name`, default: the directory's name), version `1.0.0`, and one destination per loaded CLI profile, computed from the profile templates (`clis/*.toml`); the pack's CLI list is the union of these destinations. An existing pack is refused unless `--force` is passed.

```bash
# my-pack/
# ├── skills/tdd/SKILL.md
# ├── agents/debugger.toml
# ├── rules/no-panic.md
# └── commands/review.toml
rapm add ./my-pack
```

## Removing a pack from the library: `rapm remove <name>`

```bash
rapm remove <name>
```

Deletes `<user library>/<name>`. Only the writable user library is ever touched: removing a pack that exists only in a system location (RPM/DEB, `./packs`) fails with exit `1`.

## Headless CLI (scripts / CI)

The fundamental commands, without a terminal — served by the same service as the TUI, with the same rules (destination validation, `packs_index.toml`, exit codes):

```bash
rapm install <pack> [--dest <dir>] [--force]
rapm uninstall <pack> [--dest <dir>]
rapm add <dir> [--name <name>] [--force]
rapm remove <name>
rapm help
```

- `install` copies the pack's items into `--dest` (default: the current directory), records the installation in `<dest>/packs_index.toml`, and refuses existing destinations unless `--force` is passed. The target CLI is the `default_cli` of the [configuration file](#configuration-file), else the pack's first declared CLI.
- `uninstall` removes exactly the assets recorded in the index, then drops the index entry.
- `add` / `remove` manage the pack library (see above).
- Exit codes: `0` success, `1` runtime error (unknown pack, destination already exists without `--force`, no recorded install…), `2` usage error.

## The screen

Three sections, all in shared rounded boxes (rusty-console design):

- **Header**: application title + selected pack.
- **Split screen**: **Packs** (left top, available packs grouped by target CLI) / **Installed** (left bottom, packs installed in the explorer's current directory) / **Explorer** (right, host file tree — folders in blue, listed first).
- **Footer**: the two always-available commands — `Tab: Switch panel | Space: Help`. All contextual keys live in the Space help.

The focused panel has a cyan border.

## Keys

### Packs panel (left top focus)

| Key | Action |
|---|---|
| `Tab` | Move focus to the installed panel |
| `↑`/`↓` or `k`/`j` | Move in the pack list |
| `A` | **Install** the selected pack into the explorer's current directory (refreshes the explorer, shows a success popup) |
| `P` | **Preview** the install plan (dry-run popup: one line per item with status `ok` / `collision` / `hors index`; writes nothing, any key closes) |
| `V` | **View**: open the pack viewer on the pack's `pack.toml` (`.md`, `.js`, `.toml` files, with syntax highlighting) |
| `Space` | Contextual help |
| `Ctrl-C` | Quit |

### Installed panel (left bottom focus)

Lists the packs recorded as installed in the explorer's current directory (see [Installation index](#installation-index)). A pack listed here no longer appears in the Packs panel.

| Key | Action |
|---|---|
| `Tab` | Move focus to the explorer |
| `↑`/`↓` or `k`/`j` | Move in the installed list |
| `U` | **Uninstall** the selected pack: removes every file it installed (confirmation dialog), then forgets the installation |
| `Space` | Contextual help |
| `Ctrl-C` | Quit |

### Explorer panel (right focus)

| Key | Action |
|---|---|
| `Tab` | Move focus to the packs |
| `↑`/`↓` or `k`/`j` | Move in the list |
| `Enter` | Open the selected folder |
| `Backspace` | Go to the parent folder |
| `R` | Create a folder in the current directory (input dialog: name, `Enter` validates, `Esc` cancels) |
| `M` | Rename the selected folder/file (input dialog prefilled with the current name) |
| `D` | Delete the selected folder/file (confirmation dialog; `Shift-D` forces a non-empty folder) |
| `Space` | Contextual help |
| `Ctrl-C` | Quit |

### Installation

`A` on the packs panel installs immediately into the **explorer's current directory** and refreshes the explorer. Navigate the explorer first (Tab, Enter/Backspace) to choose the target directory. `P` shows the install plan as a popup before installing (no write).

The install targets the **`default_cli` of the configuration file** when the pack declares it, else the **pack's first declared CLI** (all shipped packs declare `["vibe", "gemini"]` except `vibe-wise-commands`, which is Gemini-only). If a destination already exists, the install is refused (no silent overwrite) and an error message is shown.

### Installation index

Every successful install is recorded in a `packs_index.toml` file at the root of the target directory:

```toml
[[packs]]
name = "tdd"
version = "1.0.0"
cli = "vibe"
paths = [".agents/skills/tdd", ".vibe/agents/tdd-orchestrator.toml"]
```

- One entry per installed `(pack, CLI)` couple; reinstalling overwrites the entry.
- `paths` lists the paths (relative to the target root) actually copied at install time. They are the source of truth for uninstalling.
- `U` on the Installed panel removes those paths (files or whole directories) and drops the entry. Assets already deleted by hand are tolerated.
- The index is written **after** the files are copied: a failed install leaves no entry.
- `packs_index.toml` is reserved: no pack item may install to this path, and deleting it by hand loses the tracking (uninstall becomes impossible from the TUI).

### Uninstalling

`U` on the Installed panel asks for confirmation, then removes every asset listed in the pack's index entry (files and directories, recursively) and drops the entry. The pack becomes available again in the Packs panel. Uninstalling is idempotent: assets missing on disk (deleted by hand since the install) are skipped.

### Viewer (after `V`)

The viewer opens on the pack's `pack.toml` manifest.

| Key | Action |
|---|---|
| `Tab` | Switch between the file list and the content |
| `Enter` | Show the selected file's content |
| `↑`/`↓` | Scroll / change file |
| `Backspace` | Go back to the file list |
| `Esc` | Close the viewer |

## Pack format

A pack is a directory in `packs/` containing a `pack.toml` manifest:

```toml
name = "tdd"
version = "1.0.0"
description = "TDD discipline pack"
cli = ["vibe", "gemini"]

[[items]]
kind = "skill"           # skill | agent | command
source = "skills/tdd"   # path in the pack (file or directory)
[items.destinations]
vibe = ".agents/skills/tdd"
gemini = ".gemini/skills/tdd"
```

- `name`: lowercase, digits, `-`, `_`; starts with a letter.
- `version`: semver `MAJOR.MINOR.PATCH` (`-rc.1` suffix allowed).
- `cli`: any CLI id with a loaded profile (see below); a pack must target at least one.
- `destinations`: paths relative to the target directory chosen at install time; absolute or upward (`..`) paths are rejected. **Optional** for a profiled CLI: when omitted, the destination is computed from the CLI profile (`{name}` = item name).

## CLI profiles (`clis/`)

The destination layout of each CLI lives in a profile file `clis/<id>.toml` (shipped in `clis/`, loaded from the user library `$XDG_DATA_HOME/rapm/clis` first, then `clis/` next to the binary, in the cwd, and `/usr/share/rusty-agentic-pack-manager/clis` — first match wins, so a user profile overrides a shipped one):

```toml
[cli]
name = "claude"
label = "Claude Code"
[defaults]
skill     = ".claude/skills/{name}/SKILL.md"   # file source
skill_dir = ".claude/skills/{name}/"            # directory source (optional)
agent     = ".claude/agents/{name}.md"
command   = ".claude/commands/{name}.md"
```

- `{name}` = the item name: the source directory's name, or its parent directory for a file source (`skills/tdd/SKILL.md` -> `tdd`).
- `{file}` = the source file name, also available in templates.
- A pack may declare `cli = ["claude"]` with items **without** `[items.destinations]`: destinations are computed from the profile at install time. Explicit destinations always win (retrocompatible with existing packs).
- Shipped profiles: `vibe`, `gemini`, `claude`, `copilot`, `opencode`, `codex`, `cursor`.

## Embedded packs

16 packs shipped: `tdd`, `code-review`, `debugging`, `security`, `planning`, `brainstorming`, `quality-gate`, `prd`, `work-slicing`, `karpathy`, `parallel-agents`, `focus-adhd`, `rusty-agnostic`, `learning` (feynman-learning, deep-research, fact-check, curriculum-planner), `vibe-wise` (learning mode skills with scripts), `vibe-wise-commands` (Gemini CLI slash commands for VibeWise).
