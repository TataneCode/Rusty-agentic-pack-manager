---
name: code-review
description: "Rigorous code review: use to request a review before merging (diff analysis, finding severity, blocking points) or to handle review feedback (verify every finding before implementing it, push back with arguments when justified)."
license: MIT
---

# Code Review — Code review

Two uses: **requesting** a review, and **handling** feedback.

## Part A — Requesting a review

### When

- Task finished, major feature, before any merge/PR.
- Local verification has passed (see `verification-gate`).

### How

1. **Set the stage**: show the diff (`git diff`), not the whole file. Summarize what you changed and why in 3–5 lines.
2. **Self-review first**: reread your own diff as a stranger. Fix what you see before asking.
3. **Guide the reviewer**: list the points where you want an opinion (architecture decision, risky area, adopted trade-off).
4. **Request the structured review**:

```markdown
## Review requested: [title]

### Context
[2–3 lines: what and why]

### Diff
[git diff or link]

### Verifications done
- [command] → [result]

### Points to dig into
1. [decision 1]
2. [risky area 2]
```

### What the review must produce

Every finding ranked by severity:

| Severity | Meaning | Action |
|---------|---------------|--------|
| **Blocker** | Bug, security, regression, data loss | Fix before merge |
| **Major** | Bad practice, technical debt, readability | Fix or explicit ticket |
| **Minor** | Style, naming, comment | Fix if quick, otherwise note |
| **Question** | Choice to validate, missing information | Answer, do not implement |

Rule: an unresolved **Blocker** = no merge.

## Part B — Handling a review

### The rule

**Verify before agreeing.** Review feedback is not an order. Understand it, evaluate it, then respond.

### The response pattern

For every finding:

1. **Read it entirely.** Do not answer only the first line.
2. **Verify the claim.** Is the reviewer right? Reproduce if technical (read the code, test).
3. **Classify**: agree / disagree / unclear / out of scope.
4. **Respond explicitly**: no silent "ok", no silent implementation.

### Forbidden responses

- "Good catch, I'll fix it" (without having verified it is true).
- Fixing without reading the finding to the end.
- Ignoring a Blocker finding without saying why.
- "It's not a big deal" without an argument.

### When to push back

Push back when:
- The claim is factually false (with supporting evidence: code, test, doc).
- The finding is out of the branch's scope (propose a separate ticket).
- The "problem" is a style preference without impact (cite the repository convention if it exists).
- The change would break something else (explain what, with evidence).

Formulate constructive pushback: "I disagree because [evidence]. However [counter-proposal]. Can you confirm?"

### Implementation order for feedback

1. Blockers first.
2. Majors next.
3. Minors if quick.
4. Questions: answer, do not implement.
5. Out of scope: note them, do not do them in this branch.

After every fix: rerun the related verification (see `verification-gate`), then answer the review thread point by point.

## Pitfalls

- **Performative agreement**: saying yes to everything to go fast. Every unkept "yes" costs more later.
- **Facade review**: requesting a review without a diff or context.
- **Merging with an open Blocker**: never.
