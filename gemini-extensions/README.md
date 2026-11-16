# Gemini CLI extensions — conversions from Claude Code plugins

One Gemini CLI extension per source plugin, in the standard extension layout
(`gemini-extension.json` manifest, `commands/` custom commands in TOML,
`skills/` in the open Agent Skills format, optional context file).

| Extension | Source plugin | Conversion summary |
|---|---|---|
| `graphify/` | [Graphify-Labs/graphify](https://github.com/Graphify-Labs/graphify) | **Nothing to convert**: native Gemini CLI support already exists upstream (`graphify install --platform gemini`; the repo ships its own `gemini-extension.json` + `GEMINI.md`). See `graphify/README.md`. |
| `ponytail/` | [SofDev007/claude_plugins → ponytail](https://github.com/SofDev007/claude_plugins/tree/main/ponytail) | 6 skills (frontmatter cleaned to `name` + `description`), 6 custom commands (`commands/*.toml`, already Gemini TOML), `AGENTS.md` as the context file. Claude/Codex hooks (mode tracking, statusline) have no Gemini equivalent and are dropped — the context file is the always-on mechanism. |
| `rtk/` | [pleaseai/claude-code-plugins → rtk](https://github.com/pleaseai/claude-code-plugins/tree/main/plugins/rtk) | The `rtk` binary is used as-is. The Claude `PreToolUse` hook is ported to a Gemini `BeforeTool` hook on `run_shell_command` (`hooks/hooks.json` + `hooks/rtk-rewrite.sh`), the skill is cleaned/adapted, and `GEMINI.md` instructs running commands through `rtk`. Commands `/rtk:gain`, `/rtk:discover`, `/rtk:proxy`. |
| `ui-ux-pro-max/` | [nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) | The `ui-ux-pro-max` skill folder copied verbatim (SKILL.md + `references/` + `scripts/` + `data/`); `${CLAUDE_PLUGIN_ROOT}/...` script paths replaced by skill-root-relative paths. |

## Install / test

```bash
gemini extensions install --path=./ponytail     # or ./rtk, ./ui-ux-pro-max
# development:
cd ponytail && gemini extensions link .
```

Then in a Gemini CLI session: `/skills list` to check skills, `/commands list`
to check custom commands, `/hooks panel` for the rtk hook.

## Conversion rules applied

- `skills/<name>/SKILL.md`: kept `name` + `description` frontmatter only; dropped
  plugin-specific fields (`argument-hint`, `license` — the license file is kept
  at the extension root).
- `commands/*.md` → `commands/<group>/<cmd>.toml` with `description` + `prompt`
  (`{{args}}` substitution, `!{...}` shell execution).
- Hooks: Gemini extensions support `hooks/hooks.json` with a `BeforeTool`
  event, so rtk's command-rewrite hook was ported (rewrites only — Gemini has
  no auto-allow for `BeforeTool`, so its native confirmation still applies).
  ponytail's hooks (SessionStart/SubagentStart/UserPromptSubmit node scripts)
  target events Gemini does not support; they are replaced by the `AGENTS.md`
  context file and dropped.
- `${CLAUDE_PLUGIN_ROOT}` → `${extensionPath}` in manifests/hooks; in skill
  bodies, replaced by paths relative to the skill root (the directory of the
  `SKILL.md`), since `${extensionPath}` is not substituted inside skills.
- Mentions of Claude Code tooling in prompts/skills were replaced by their
  Gemini CLI equivalents (`/plugin` → `gemini extensions update`, etc.).

## Licenses

Each extension keeps its upstream license file where the source ships one
(ponytail: MIT, ui-ux-pro-max: MIT). rtk and graphify are MIT upstream
(rtk-ai/rtk, Graphify-Labs/graphify).
