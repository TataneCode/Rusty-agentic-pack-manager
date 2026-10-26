---
name: brainstorming
description: "Use before any creative work — new feature, component, behavior change, or planning anything: first frame the real need with the user, before proposing any solution."
license: MIT
---

# Brainstorming — Frame before building

## The rule

**Do not propose a solution before understanding the problem.** The sequence is: understand → question → play back → frame → only then, design.

## Phase 1 — What you actually know

Before asking questions, check what you already know:

1. Read the relevant code/doc/context. Do not ask what the repository answers.
2. Note what is known, what is uncertain, what is a constraint.

## Phase 2 — The first message

Your first message asks questions, no solution. Target:

- **The real need**: why now? What problem does this solve, for whom?
- **The constraints**: time budget, compatibility, repository conventions, out of scope.
- **The success criteria**: how will we know it succeeded?
- **The non-goals**: what we will NOT do, explicitly.

Ask a small number of questions, grouped. Not 15 questions at once.

## Phase 3 — The conversation

- One question per turn if answers are dense.
- Restate what you understood to validate ("To summarize: ... Is that right?").
- Push back when an idea contradicts a constraint or a good practice — with arguments, not an opinion.
- If the user drifts toward implementation, bring it back to the need: "The how comes after the what."

## Phase 4 — Playback

Before moving to design, play back the framing:

```markdown
## What we are building
[One sentence]

## For whom and why
[One sentence]

## Success criteria
- [verifiable criterion 1]
- [verifiable criterion 2]

## Out of scope
- [what we are not doing]

## Constraints
- [constraint 1]
```

Have this framing validated. Only proceed on an explicit "yes" (or a correction you integrate).

## Phase 5 — Sizing the work

Estimate in concrete units (minutes / hours / days), not "a bit of work". Say what fits in one session and what does not.

## Phase 6 — The written design

Only now, propose the solution:

1. **Retained approach** + **rejected approaches** (and why, one line each).
2. **Architecture/shape**: files, responsibilities, data flows.
3. **Verification plan**: what to run to prove it works.
4. **Risks**: what can block, and the plan B.

Finish with: "Validate this design, or tell me what is blocking."

## Red flags

- You proposed a solution in your first message.
- You did not read the existing code before questioning.
- The framing was not validated and you design anyway.
- The user says "whatever, do the simplest" → that is not a blank check, it is a signal to reduce scope, not to skip framing.

## Forbidden rationalizations

| Rationalization | Reality |
|-----------------|---------|
| "The need is obvious" | The user's obviousness is not yours. 2 questions confirm. |
| "Brainstorming slows us down" | A bad start costs 10× a good framing. |
| "I propose, they will correct" | They will correct afterwards, on written code. |
