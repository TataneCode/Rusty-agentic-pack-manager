# User manual — rapm

`rapm` is the TUI for managing agentic packs: list, view, install and uninstall sets of skills, agents and commands in a repository, for a target CLI (Vibe CLI, Gemini CLI). It also ships a CLI to create packs in your personal library: `rapm pack create`.

## Startup

```bash
rapm
```

The application starts in the current directory. Packs are searched in:

1. **User library**: `$XDG_DATA_HOME/rapm/packs/` (or `~/.local/share/rapm/packs/`; `%APPDATA%\rapm\packs` on Windows) — writable, this is where `rapm pack create` writes
2. `<binary directory>/packs/`
3. `./packs` (current directory)
4. `/usr/share/rusty-agentic-pack-manager/packs/` (RPM/DEB system install)

The first location containing a pack wins: a pack in the user library overrides a system pack of the same name. Packs created while the TUI is running appear after the next action (the list is reloaded on every action).

## Creating a pack from the CLI

```bash
rapm pack create [options]
```

Builds a pack from any list of files or directories and writes it into the user library. Items are copied under a canonical layout — `<kind>s/<name>/` (`skills/`, `agents/`, `commands/`):

```bash
rapm pack create \
  --name my-pack \
  --skill ./skills/tdd --vibe-dest .agents/skills/tdd/ --gemini-dest .gemini/skills/tdd/ \
  --command ./commands/review.toml --vibe-dest .vibe/commands/review.toml --gemini-dest .gemini/commands/review.toml
```

| Option | Meaning |
|---|---|
| `--name <name>` | Pack name (lowercase, digits, `-`, `_`); prompted interactively if omitted (required — a missing name without a terminal is an error) |
| `--version <version>` | Semver version (default: `1.0.0`) |
| `--description <text>` | Description; prompted if omitted (empty by default) |
| `--cli <vibe,gemini>` | Target CLIs (default: union of the items' destinations) |
| `--force` | Replace an existing pack of the same name |
| `--skill\|--agent\|--command <path>` | Add an item (file or directory); repeatable |
| `--vibe-dest <dest>` / `--gemini-dest <dest>` | Install destination for that CLI; applies to the last item |

Rules: every item needs a destination for every declared CLI; two sources of the same kind must not share a file/directory name (they would collide in the canonical layout); an existing pack is refused unless `--force` is passed. Destinations are validated at creation with the same rule as at install time: relative paths only — absolute paths (`/etc/...`), upward escapes (`../...`) and root-only destinations (`.`, `/`) are rejected, so a created pack can always be installed. Exit codes: `0` success (and `--help`), `2` usage error (unknown option, missing value, missing `--name` without a terminal), `1` any other error. Accepted limits: a pack created while the TUI runs appears after the next action; `--force` is best-effort (a crash mid-swap can leave a `<name>.old-<pid>` directory).

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

`A` on the packs panel installs immediately into the **explorer's current directory** and refreshes the explorer. Navigate the explorer first (Tab, Enter/Backspace) to choose the target directory.

The install targets the **pack's first declared CLI** (all shipped packs declare `["vibe", "gemini"]` except `vibe-wise-commands`, which is Gemini-only, so Vibe destinations are used; a CLI picker is planned for v1.1). If a destination already exists, the install is refused (no silent overwrite) and an error message is shown.

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
- `destinations`: paths relative to the target directory chosen at install time; absolute or upward (`..`) paths are rejected.

## Embedded packs

16 packs shipped: `tdd`, `code-review`, `debugging`, `security`, `planning`, `brainstorming`, `quality-gate`, `prd`, `work-slicing`, `karpathy`, `parallel-agents`, `focus-adhd`, `rusty-agnostic`, `learning` (feynman-learning, deep-research, fact-check, curriculum-planner), `vibe-wise` (learning mode skills with scripts), `vibe-wise-commands` (Gemini CLI slash commands for VibeWise).
