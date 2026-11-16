# RTK (Rust Token Killer) — token-optimized shell output

This extension reduces LLM token consumption by routing shell commands through RTK.

- Optional `BeforeTool` hook: the pack ships `skills/rtk-optimization/hooks/rtk-rewrite.sh`, which rewrites every `run_shell_command` call through `rtk rewrite` before execution whenever an RTK equivalent exists. Packs have no hook slot, so wire it manually in `~/.gemini/settings.json` (Gemini CLI):

  ```json
  { "hooks": { "BeforeTool": [ { "matcher": "run_shell_command", "hooks": [ { "name": "rtk-rewrite", "type": "command", "command": "bash \"$GEMINI_PROJECT_DIR/.gemini/skills/rtk-optimization/hooks/rtk-rewrite.sh\"", "timeout": 5000 } ] } ] } }
  ```

  It requires `rtk >= 0.23.0` and `jq` on `PATH`; if either is missing it warns once on stderr and passes commands through unchanged. Vibe CLI has no hook support: use the `rtk <cmd>` prefix or the commands below. Note the hook only rewrites — Gemini's normal confirmation still applies (no auto-allow).
- When issuing shell commands, prefer `rtk <command>` (e.g. `rtk git status`) so the output is compressed even outside the hook.
- Analytics: `/rtk-gain` shows measured token savings, `/rtk-discover` finds commands that should have gone through RTK, `/rtk-proxy <cmd>` runs a command raw for debugging.
- Disable RTK for a single invocation with `RTK_DISABLED=1 <command>`. Config lives in `~/.config/rtk/config.toml` (view with `rtk config`, template via `rtk config --create`); per-project overrides go in `.rtk/filters.toml` at the project root.
