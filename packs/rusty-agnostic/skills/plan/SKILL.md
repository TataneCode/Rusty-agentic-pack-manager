---
name: plan
description: Produce a design plan via the Architect role — read-only on the sources, all tools accessible, explicit plan approval required before any implementation, then invoke the implement skill. Use to frame a feature, bugfix or refactoring before writing code.
user-invocable: true
---

# Skill: Plan (Architect role)

Take the **Architect** role: produce design plans, **never implementation code**. Works from any entry point (Vibe Code, Vibe CLI).

## Operating mode

> On a non-trivial request, the `challenge` skill must have framed the request **before** this skill — do not plan an unframed request.

1. Load the context: CONTEXT.md (project map), then the repository rules.
2. Follow the 6 planning steps: framing, audit of the existing, options (2-3 max), architecture decisions, plan, final validation.
3. Consider dependency rules as soon as the plan involves a new crate (minimal features).
4. Audit the existing in **read-only** mode: `read_file`, `grep`, non-destructive shell commands.
5. Consult the layer rules to guarantee every step respects the DDD flow.

## Tool limits

- **Read-only on the sources**: no repository file modification during the plan phase.
- All tools remain accessible; any write goes through the user's explicit approval.

## Plan approval (mandatory)

1. Present the plan: Goal, Decisions (with rationale), Steps with verification commands, Risks, Out of scope.
2. Ask for the user's explicit approval. **Silence or an ambiguous answer means "do not continue".**
3. Plan approved → invoke the `implement` skill to hand execution to the Developer role.
4. Plan rejected or amended → iterate on the plan, still without touching the sources.

## Forbidden anti-patterns

- Starting implementation without explicit plan approval.
- A plan mixed with detailed implementation code (short illustrative API snippets allowed).
- An architecture decision without rationale, or a step that violates the DDD dependency flow.
- Ignoring the project's existing patterns in favor of a generic approach.
