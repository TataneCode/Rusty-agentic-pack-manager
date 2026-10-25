---
name: work-slicing
description: "Use when a plan or spec exists and must be broken into actionable pieces: produces a list of vertical slices, each autonomous and verifiable, with declared dependencies."
license: MIT
---

# Work Slicing — Slicing the work

Turns a plan or spec into a set of actionable slices. It is the step between "we know what we are building" and "someone can pick up a piece".

## When to use

- A plan or spec exists and must be broken into queueable work.
- A task is too big for a single session and must be shared.
- Several people or sessions will work in parallel and need clear boundaries.

## When not to use

- Decisions are still open: slicing undecided work produces slices that will be reopened. Use `brainstorming` or `thought-partner` first.
- The work fits in one session: one slice, or none.
- You want parallel execution right now: use `parallel-agents`.

## Method

### 1. Slice vertically

Every slice cuts a narrow but **complete** path through all the layers it touches — schema, API, UI, tests. It is NOT a horizontal slice of a single layer.

- A finished slice is demonstrable or verifiable on its own.
- Every slice fits in a fresh context window (a few files, not half a project).

### 2. Declare what blocks what

For every slice, list the slices it depends on. A slice with no dependency is ready. A cyclic dependency blocks both — re-cut.

### 3. Produce the output

For every slice, one block:

```markdown
## Slice N: [title]
- Goal: [what works at the end, in one sentence]
- Files: `path/a`, `path/b`
- Depends on: Slice M (or "none — ready")
- Verification: `[command]` → [expected result]
- Estimate: [concrete time]
```

### 4. Order

Give the execution order: slices with no dependency first, then by dependency chain.

## Stopping rules

- If slicing reveals an undecided point, stop and return to the framing phase.
- If a slice grows along the way, re-cut it rather than letting it bloat.
- Do not slice beyond usefulness: 5–8 slices for a medium project, not 30 micro-tasks.

## Output contract

- Numbered list of slices, each with goal, files, dependencies, verification, estimate.
- Explicit execution order.
- What is deliberately out of scope.

## Verification

- Every slice reads standalone: a reader picking it without the rest understands what to do and how to verify.
- The sum of the slices covers the initial plan with no hole and no overlap.
