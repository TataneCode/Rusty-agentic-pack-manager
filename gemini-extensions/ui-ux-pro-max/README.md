# ui-ux-pro-max — Gemini CLI extension

Converted from
[nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill)
(skill v2.15.0, MIT).

## Contents

- `gemini-extension.json` — manifest (skills are auto-discovered from
  `skills/`; no context file needed).
- `skills/ui-ux-pro-max/` — the skill folder copied from the source repo:
  `SKILL.md`, `references/` (quick-reference.md, pro-rules.md), `scripts/`
  (`search.py` + helpers) and `data/` (the searchable CSV/JSON catalogs).

## Conversion notes

- Frontmatter already matched the Agent Skills standard (`name` +
  `description`); unchanged.
- `${CLAUDE_PLUGIN_ROOT}/.claude/skills/ui-ux-pro-max/scripts/search.py` was
  replaced by `scripts/search.py` — paths are now relative to the skill root
  (the directory containing the `SKILL.md`, which Gemini CLI grants access to
  when the skill activates). `${extensionPath}` is only substituted in
  `gemini-extension.json` and `hooks/hooks.json`, not inside skill bodies.
- The upstream `uipro init --ai gemini` CLI installs the same skill into a
  project's `.gemini/skills/`; this extension is the global, project-agnostic
  equivalent. The repo also ships companion skills (`design`, `brand`,
  `design-system`, `ui-styling`, `slides`, `banner-design`) not bundled here.

## Install / test

```bash
gemini extensions install --path=./ui-ux-pro-max
# dev: cd ui-ux-pro-max && gemini extensions link .
```

In a session: `/skills list` should show `ui-ux-pro-max`; ask for a UI design
task and the skill activates; `python scripts/search.py "dashboard" --domain ux`
run from the skill root returns catalog entries (requires Python 3, no deps).
