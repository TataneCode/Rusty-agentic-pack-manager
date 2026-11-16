# RTK (Rust Token Killer) — token-optimized shell output

This extension reduces LLM token consumption by routing shell commands through RTK.

- A `BeforeTool` hook (`hooks/hooks.json`) intercepts every `run_shell_command` call and rewrites it through `rtk rewrite` before execution whenever an RTK equivalent exists. It requires `rtk >= 0.23.0` and `jq` on `PATH`; if either is missing it warns once on stderr and passes commands through unchanged.
- When issuing shell commands, prefer `rtk <command>` (e.g. `rtk git status`) so the output is compressed even outside the hook.
- Analytics: `/rtk:gain` shows measured token savings, `/rtk:discover` finds commands that should have gone through RTK, `/rtk:proxy <cmd>` runs a command raw for debugging.
- Disable RTK for a single invocation with `RTK_DISABLED=1 <command>`. Config lives in `~/.config/rtk/config.toml` (view with `rtk config`, template via `rtk config --create`); per-project overrides go in `.rtk/filters.toml` at the project root.
