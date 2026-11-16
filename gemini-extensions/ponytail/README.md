# ponytail — Gemini CLI extension

Converted from the Claude Code plugin
[SofDev007/claude_plugins → ponytail](https://github.com/SofDev007/claude_plugins/tree/main/ponytail)
(v4.8.4, MIT).

Lazy senior dev mode: forces the simplest, shortest solution that actually
works — YAGNI, stdlib first, no unrequested abstractions.

## Contents

- `gemini-extension.json` — manifest; `contextFileName: AGENTS.md` loads the
  always-on ponytail rules into every session.
- `AGENTS.md` — permanent context (the ponytail ladder and rules).
- `commands/*.toml` — `/ponytail`, `/ponytail-review`, `/ponytail-audit`,
  `/ponytail-debt`, `/ponytail-gain`, `/ponytail-help`.
- `skills/*/SKILL.md` — the six ponytail skills (Agent Skills format,
  frontmatter reduced to `name` + `description`).

## Conversion notes

- The upstream plugin already shipped `commands/*.toml` in Gemini CLI format
  and a `gemini-extension.json`; this extension assembles them into a clean
  standalone directory and cleans the skill frontmatter.
- The Claude/Codex hooks (`hooks/*.js`: SessionStart activation, subagent
  injection, mode tracker, statusline) fire on events Gemini CLI does not
  support. They are intentionally dropped: in Gemini the `AGENTS.md` context
  file provides the always-on behavior and `/ponytail <level>` switches mode
  for the current session (no cross-session persistence, no statusline).
- `skills/ponytail-help/SKILL.md` was adapted: update flow now points to
  `gemini extensions update ponytail`, and mode persistence notes reflect the
  hook-less behavior.

## Install / test

```bash
gemini extensions install --path=./ponytail
# dev: cd ponytail && gemini extensions link .
```

In a session: `/ponytail ultra`, `/ponytail-review`, `/skills list`.
