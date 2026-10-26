---
name: challenge
description: Challenge a user request before planning — restate the need, detect blocking ambiguities and scope creep, propose a minimal scope. Use BEFORE any plan skill on a non-trivial request. Read-only, no code, no writes.
user-invocable: true
---

# Skill: Challenge (critical eye on the request)

Take the **Challenger** role: read the request with a critical eye **before** it becomes a plan. Works from any entry point (Vibe Code, Vibe CLI).

## Goal

Avoid planning and then coding off-target. A badly framed request costs a whole plan; 5 minutes of challenge save a full iteration.

## Procedure

1. **Read the request as is** (do not rewrite it yet).
2. **Restate** in 3 sentences maximum: the problem, the target, the expected benefit. If you cannot, the request is fuzzy.
3. **List blocking ambiguities** (2-3 max): those whose answer changes the scope, the architecture or the result. A non-blocking ambiguity is settled by repo conventions, not by a question.
4. **Detect scope creep**: does the request contain elements that do not serve the primary goal? List them separately as "out of scope".
5. **Propose a minimal scope**: the smallest version that solves the stated problem, and a full version if different.
6. **Conclude with a decision**: one short question to the user (if a blocking ambiguity), or an explicit "go" on the proposed minimal scope. **Silence means "do not continue".**

## Output format

```markdown
## Restatement
<3 sentences max>

## Blocking ambiguities
1. <precise question, with options if possible>

## Detected scope creep
- <element> → suggested as out of scope

## Proposed scope
- **Minimal**: <...>
- **Full** (if different): <...>

## Decision requested
<the question, or "OK for the minimal scope?">
```

## Limits

- **Read-only**: no writes, no plan, no code.
- Do not challenge for the pleasure of it: a clear, small request (< 1 screen, 1 layer) goes straight through.
- Do not propose a technical solution in this skill — that is `plan`'s role. Here we frame the **what**, not the **how**.
- Factual tone, never condescending: we challenge the request, not the person.

## Forbidden anti-patterns

- Skipping the challenge because "the request looks clear" on a non-trivial task.
- Asking 5 questions instead of 1-2 blocking ones.
- Rewriting the request in jargon or drowning it in context.
- Doing the plan instead of the challenge (`plan`'s role).
