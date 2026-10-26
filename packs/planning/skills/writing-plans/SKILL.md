---
name: writing-plans
description: "Use when a spec or requirements exist for a multi-step task, before touching code: write a structured implementation plan, with numbered tasks, every step with its verification command."
license: MIT
---

# Writing Plans — Writing a plan

## When to use

- Multi-step task (more than ~3 non-trivial actions).
- Feature, migration, multi-file refactoring.
- When the user explicitly asks for a plan.

Do not use for: a one-file fix, a question, a search.

## Plan structure

```markdown
# Plan: [Feature name]

## Goal
One sentence: what the reader will be able to do afterwards.

## Prerequisites
- Required tools/versions/dependencies.

## Tasks

### Task 1: [Component]
- Files touched: `path/to/file`
- Steps:
  1. [action]
  2. [action]
- Verify: `[command]` → [expected result]

### Task 2: [Component]
...

## Order and dependencies
Task 2 depends on task 1, etc.

## Out of scope
What this plan does NOT do (to cut off drift).

## Review
Vigilance points for code review.
```

## Size rules

- **One task = one component** (one file, one module, one responsibility).
- **One step = one verifiable action.** If a step cannot be verified, it is too big — split it.
- Every step ends with "Verify: [command] → [expected]" (see `verification-gate`).
- No cascading "and then" inside a step: that is two steps.

## Granularity

- Too big ("implement auth") → nobody can pick it up, everything lives in one head.
- Too fine ("create file X") → noise, not a plan.
- Just right: a task takes a few minutes to an hour, reads standalone, verifies standalone.

## Before writing the plan

1. Verify the request is clear. If a structural choice is open (approach A or B), ask the question or present both options with a recommendation — do not write a plan on a fuzzy foundation.
2. Read the relevant code (do not plan in a vacuum).
3. List global constraints (repository style, no new dependency, compatibility...).

## After the plan

- Have it validated by the user before executing.
- Store the plan in the repository if the project expects it (e.g. `docs/plans/<date>-<topic>.md`).
- Execute task by task, checking off as you go. Do not skip verifications.

## Plan self-review (before presenting it)

- [ ] Every step has its verification.
- [ ] Every task stands alone (files named).
- [ ] The order respects dependencies.
- [ ] The out-of-scope section is explicit.
- [ ] No "we'll see along the way".
