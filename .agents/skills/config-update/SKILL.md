---
name: config-update
description: Evolve the agent configuration of this repository — add/modify a skill, a layer rule, a Vibe CLI agent, a slash command; keep consistency (no duplication, single sources of truth, up-to-date references). Use when touching AGENTS.md, CONTEXT.md, rules/, .agents/skills/, agents/ or .vibe/.
user-invocable: true
---

# Skill: Agent configuration update

Evolve the repository's vibe-coding configuration (skills, rules, agents, commands) **without breaking consistency** and while keeping the context size under control. Works from any entry point (Vibe Code, Vibe CLI).

## Guiding principles

1. **One source of truth per topic** — never duplicate an instruction in two files. If content already exists, reference it, do not copy it.
2. **Progressive disclosure** — a verbose skill splits: short procedure in `SKILL.md`, details in `references/<topic>.md` loaded on demand.
3. **Context has a cost** — every line of `AGENTS.md` is injected into every session. Trim > enrich.
4. **Vibe Code / Vibe CLI duality** — a skill (`.agents/skills/`) covers both; a TOML agent (`.vibe/agents/`) covers only Vibe CLI and must have a skill counterpart or its role documented in `agents/`.

## Procedure

### 1. Frame the change
- Which topic evolves (skill, rule, agent, command)? Which concrete problem does it solve (tokens, duplication, friction)?
- Read [CONTEXT.md](../../../CONTEXT.md) and [AGENT_ARCHITECTURE.md](../../../AGENT_ARCHITECTURE.md) to understand the overall articulation.

### 2. Choose the right place (anti-duplication)

| The content is… | It lives in |
|---|---|
| A layer or cross-cutting rule (normative, stable) | `rules/<layer>.md` or `rules/testing.md` / `rules/quality.md` |
| An operational procedure (how to do a task) | `.agents/skills/<name>/SKILL.md` |
| An agent role (persona) | `agents/<role>.md` + `.vibe/agents/<role>.toml` + `.vibe/prompts/<role>.md` |
| Navigation / project context | `CONTEXT.md` |
| Startup routing | `AGENTS.md` (routing table only) |
| An overly verbose procedure detail | `references/` of the relevant skill |

### 3. Apply the change

- **New skill**: `.agents/skills/<name>/SKILL.md` with frontmatter (`name`, `description` with triggers, `user-invocable: true` if slash command). If it has a CLI agent counterpart: `agents/<role>.md` + `.vibe/agents/<role>.toml` (adapted safety) + `.vibe/prompts/<role>.md`.
- **New Vibe CLI agent role**: `.vibe/agents/<role>.toml` — `safety = "safe"` if read-only, `"neutral"` if writes possible (restrictive bash allowlist, git commit/push outside the allowlist).
- **New rule**: add it to the relevant `rules/` file, reference it from `rules/README.md`, and have the relevant skills/agents point to it instead of summarizing it.
- **Modifying `AGENTS.md` or `CONTEXT.md`**: update the routing table on both sides; verify no normative content has crept back in.

### 4. Propagate references

- `grep -rn "<old-path-or-name>"` over the repository: update **all** pointers (skills, agents, prompts, docs).
- Update [AGENT_ARCHITECTURE.md](../../../AGENT_ARCHITECTURE.md) (tree + orchestration) if the structure changed.
- Never leave `.legacy/` referenced as a source by an active skill (archive only).

### 5. Verify consistency

- **Zero normative duplication**: the same instruction block exists only once (grep section titles across files).
- **Valid links**: every Markdown link points to an existing file.
- **No context regression**: `wc -l AGENTS.md` has not increased without justification; trimmed skills delegate to `references/`.
- **Confirmation gate**: every write goes through [confirmation-gate](../confirmation-gate/SKILL.md) — plan presented, explicit agreement, execution as presented.

## Forbidden anti-patterns

- Duplicating a rule in a skill instead of referencing `rules/`.
- Adding a Vibe CLI agent without a skill counterpart (both entry points must stay consistent).
- Enriching `AGENTS.md` instead of delegating to `CONTEXT.md` or `rules/`.
- Leaving a verbose skill where a `references/` would reduce context.
- Modifying the config without updating the routing table (`AGENTS.md` + `CONTEXT.md`).
