---
name: ponytail-help
description: 'Quick-reference card for all ponytail modes, skills, and commands. One-shot display, not a persistent mode. Trigger: /ponytail-help, "ponytail help", "what ponytail commands", "how do I use ponytail".'
---

# Ponytail Help

Display this reference card when invoked. One-shot, do NOT change mode,
write flag files, or persist anything.

## Levels

| Level | Trigger | What change |
|-------|---------|-------------|
| **Lite** | `/ponytail lite` | Build what's asked, name the lazier alternative in one line. |
| **Full** | `/ponytail` | The ladder enforced: YAGNI → stdlib → native → one line → minimum. Default. |
| **Ultra** | `/ponytail ultra` | YAGNI extremist. Deletion before addition. Challenges requirements before building. |

Level sticks until changed or session end.

## Skills

| Skill | Trigger | What it does |
|-------|---------|--------------|
| **ponytail** | `/ponytail` | Lazy mode itself. Simplest solution that works. |
| **ponytail-review** | `/ponytail-review` | Over-engineering review: `L42: yagni: factory, one product. Inline.` |
| **ponytail-audit** | `/ponytail-audit` | Whole-repo over-engineering audit: ranked list of what to delete. |
| **ponytail-debt** | `/ponytail-debt` | Harvest `ponytail:` shortcut comments into a tracked ledger. |
| **ponytail-gain** | `/ponytail-gain` | Measured-impact scoreboard: less code, less cost, more speed. |
| **ponytail-help** | `/ponytail-help` | This card. |

Gemini CLI, Claude Code and OpenCode use the slash-command forms above (this
Gemini extension bundles all six as commands); Codex uses `@ponytail`-style
mentions.

## Deactivate

Say "stop ponytail" or "normal mode". Resume anytime with `/ponytail`.
`/ponytail off` also works.

## Configure Default Mode

Default mode = `full`, active from session start: this extension loads its
`AGENTS.md` context file in every Gemini CLI session. Switch mode for the
current session with `/ponytail <level>`. There is no cross-session
persistence in Gemini CLI — the Claude/Codex hooks that read
`PONYTAIL_DEFAULT_MODE` or `~/.config/ponytail/config.json` have no Gemini
equivalent.

## Update

Run `gemini extensions update ponytail`, or re-install from the source path
(`gemini extensions install --path=./ponytail`; for a local checkout use
`gemini extensions link .`). Restart the session to pick up the new version.

## More

Full docs + examples: https://github.com/DietrichGebert/ponytail
