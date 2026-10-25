# User manual — rapm

`rapm` is the TUI for managing agentic packs: list, view and install sets of skills, agents and commands in a repository, for a target CLI (Vibe CLI, Gemini CLI).

## Startup

```bash
rapm
```

The application starts in the current directory. Packs are searched in:

1. `<binary directory>/packs/`
2. `./packs` (current directory)
3. `/usr/share/rusty-agentic-pack-manager/packs/` (RPM/DEB system install)

## The screen

Split screen:

- **Left panel — Packs**: the pack list, grouped by target CLI.
- **Right panel — Explorer**: the host machine file tree (folders in blue, listed first).

The action bar at the bottom shows the available keys for the focused panel. The focused panel has a cyan border.

## Keys

### Packs panel (left focus)

| Key | Action |
|---|---|
| `Tab` | Move focus to the explorer |
| `↑`/`↓` or `k`/`j` | Move in the pack list |
| `A` | **Apply**: prepare the selected pack installation (focus moves to the explorer in install mode) |
| `V` | **View**: open the pack viewer (`.md`, `.js`, `.toml` files, with syntax highlighting) |
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

### Install mode (after `A`)

Only these actions are available:

| Key | Action |
|---|---|
| `↑`/`↓` or `k`/`j` | Move in the list |
| `Enter` | Open the selected folder |
| `Backspace` | Go to the parent folder |
| `I` | **Install the pack in the explorer's current directory** |
| `Esc` | Cancel and go back to the packs |
| `Space` | Contextual help |

The installation copies every pack item to its manifest-declared destination, **under the chosen directory**. The install targets the **pack's first declared CLI** (all 13 shipped packs declare `["vibe", "gemini"]`, so Vibe destinations are used; a CLI picker is planned for v1.1). If a destination already exists, the install is refused (no silent overwrite) and an error message is shown.

### Viewer (after `V`)

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

13 packs shipped: `tdd`, `code-review`, `debugging`, `security`, `planning`, `brainstorming`, `quality-gate`, `prd`, `work-slicing`, `karpathy`, `parallel-agents`, `focus-adhd`, `rusty-agnostic`.
