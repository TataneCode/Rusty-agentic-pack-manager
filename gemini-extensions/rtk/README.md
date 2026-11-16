# rtk — Gemini CLI extension

Converted from the Claude Code plugin
[pleaseai/claude-code-plugins → rtk](https://github.com/pleaseai/claude-code-plugins/tree/main/plugins/rtk)
(v1.0.0, MIT). RTK itself is the unchanged Rust binary from
[rtk-ai/rtk](https://github.com/rtk-ai/rtk); only the plugin wrapper was
Claude-specific.

## Contents

- `gemini-extension.json` — manifest; `contextFileName: GEMINI.md`.
- `GEMINI.md` — permanent instructions: run shell commands through `rtk`,
  analytics commands, config locations.
- `hooks/hooks.json` + `hooks/rtk-rewrite.sh` — port of the Claude
  `PreToolUse` hook to Gemini's `BeforeTool` event on `run_shell_command`:
  every shell command is rewritten through `rtk rewrite` when an RTK
  equivalent exists. Difference vs Claude: Gemini's `BeforeTool` has no
  auto-allow decision, so rewrites still pass through Gemini's normal
  confirmation; deny/ask rules from `rtk rewrite` pass through to that same
  confirmation. Requires `rtk >= 0.23.0` and `jq`; fails open with a stderr
  warning otherwise.
- `commands/rtk/{gain,discover,proxy}.toml` — `/rtk:gain`, `/rtk:discover`,
  `/rtk:proxy <cmd>` (shell execution via `!{...}`).
- `skills/rtk-optimization/SKILL.md` — the RTK skill, frontmatter reduced to
  `name` + `description`, hook/init sections adapted to the Gemini extension.

## Prerequisites

```bash
# rtk >= 0.23.0 (use the explicit Git URL — plain `cargo install rtk` may
# install the unrelated "Rust Type Kit")
cargo install --git https://github.com/rtk-ai/rtk rtk
# or: brew install rtk-ai/tap/rtk
# or: curl -fsSL https://raw.githubusercontent.com/rtk-ai/rtk/master/install.sh | sh
jq --version   # jq
```

## Install / test

```bash
gemini extensions install --path=./rtk
# dev: cd rtk && gemini extensions link .
```

In a session: `/hooks panel` should list `rtk-rewrite`; run any shell command
and check it was rewritten (e.g. `git status` → `rtk git status`); `/rtk:gain`
for the savings report. Note: the hook command uses `bash`, so on Windows it
needs Git Bash / WSL on PATH.
