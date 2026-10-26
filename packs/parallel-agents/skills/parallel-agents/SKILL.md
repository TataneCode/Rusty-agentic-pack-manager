---
name: parallel-agents
description: "Use when facing 2 or more independent tasks, with no shared state and no sequential dependency: cut into focused domains, dispatch in parallel via the available sub-agents, integrate the results."
license: MIT
---

# Parallel Agents — Parallel agents

## When to use

- 2 or more **independent** tasks: no shared state, no sequential dependency, no conflict on the same files.
- Every task is big enough to justify a dedicated agent (not a 2-line change).

## When NOT to use

- The tasks step on each other (same files, same module).
- Task 2 depends on the result of task 1.
- The work fits in a few minutes sequentially — parallelism adds coordination cost.

## The pattern

### 1. Identify independent domains

For every candidate task, check:

- Disjoint files?
- No function call between the two?
- Same expected result whatever the execution order?

If any of the three is in doubt, it is not independent.

### 2. Create focused tasks

Every sub-agent receives a standalone prompt:

```markdown
## Task: [title]

### Context
[3–5 lines: the project, what already exists, the conventions]

### Goal
[One sentence: what must be true at the end]

### Scope
- Files to touch: [list]
- Out of scope: [what NOT to touch — the other parallel tasks work elsewhere]

### Expected steps
1. ...
2. ...

### Verification
`[command]` → [expected result]

### Report
Summarize: files modified, decisions made, verifications executed (commands + outputs), open points.
```

Golden rule of the prompt: the sub-agent must not need to know what the other tasks do, only what it must not touch.

### 3. Dispatch in parallel

- Launch the sub-agents in parallel (one sub-agent call per task, same turn).
- Do not do the work yourself while waiting: your role is coordination + integration.

### 4. Review and integrate

On receiving every report:

1. **Verify the claims**: were the verification commands actually executed? Reread the cited outputs (see `verification-gate`).
2. **Verify the scope**: did the sub-agent touch the other tasks' files?
3. **Integrate**: resolve merge conflicts if there is overlap (normally none).
4. **Verify globally**: run the whole project's test suite, not just one task's.

### 5. Final report

```markdown
## Parallel dispatch result
- Task A: [result] — verified: [command]
- Task B: [result] — verified: [command]
- Integration: [conflicts resolved or "none"]
- Global verification: [command] → [result]
```

## Common mistakes

| Mistake | Consequence | Fix |
|--------|-------------|-----|
| "Independent" tasks touching the same file | Merge conflicts, regression | Re-cut the boundaries before launching |
| Too-vague sub-agent prompt | Unusable result | Context + scope + verification are mandatory |
| Accepting the report without verifying | Bug shipped to production | Reread the outputs, run the global verification |
| Dispatching 5 micro-tasks | Coordination cost > gain | Keep it sequential under ~5 min |

## Adaptation per CLI

- **Vibe CLI**: sub-agents available via the `~/.vibe/agents/` config (see this repository's `agents/` folder).
- **Gemini CLI**: no declared sub-agents; parallelism happens via several CLI sessions in separate git worktrees, or sequentially with a single agent.
- Always check what the current CLI exposes before promising parallelism.
