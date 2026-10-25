---
name: systematic-debugging
description: "Use as soon as a bug, failing test or unexpected behavior appears, before proposing any fix: a 4-phase investigation — root cause, pattern analysis, tested hypothesis, then fix. Never fix without having understood."
license: MIT
---

# Systematic Debugging — Systematic debugging

## The iron rule

**Do not fix without having understood.** A fix applied to a symptom is a new bug in the making. The sequence is mandatory: root cause first, fix second.

## The 4 phases

### Phase 1 — Root cause investigation

Goal: understand why the behavior occurs, not just where.

1. **Reproduce**: write the exact steps that trigger the problem. Run them. Confirm the symptom.
2. **Read the entire error**: full message, stack trace, logs, exit code. Do not truncate.
3. **Inspect the state**: variable values at crash time, file contents, intermediate command outputs.
4. **Locate**: temporarily add logs/prints if needed, or read the code of the execution path.
5. **Ask "why" 5 times**: walk up the causal chain to the first cause (not just the nearest link).

Rule: at the end of phase 1, you must be able to say in one sentence: "It breaks because X, and X is wrong because of Y."

### Phase 2 — Pattern analysis

1. Does this bug resemble a past bug in this repository? Search (`grep`, git history).
2. Is the pattern local (one function) or systemic (several places)?
3. Are there other places with the same latent defect? List them without fixing them yet.

### Phase 3 — Hypothesis and test

1. Formulate ONE testable hypothesis for the root cause.
2. Design the minimal test that confirms or disproves it (reproduction test, request, script).
3. Run it. If the hypothesis is disproved, go back to phase 1. Do not "try things" at random.

### Phase 4 — Implementation

1. Fix the root cause, not the symptom.
2. The fix is minimal and surgical (see `karpathy-guidelines`).
3. **Verify** (see `verification-gate`): the reproduction test passes, the test suite has no regression.
4. Clean up: remove temporary logs/prints, disposable test files.
5. If the pattern was systemic (phase 2), flag the other affected sites — fix them if in scope, mention them otherwise.

## Red flags — STOP and return to the process

- You are about to modify code without having reproduced the bug.
- You changed 3 things and "maybe it works now".
- You cannot explain the root cause in one sentence.
- The "fix" consists of removing an error message or adding a `try/catch` that swallows the exception.

## Signals that the user gives you when you are drifting

- "We already tried that" → you propose a variation without a new hypothesis.
- "Why are you changing this again" → you iterate without proof.
- "That's not the problem" → you did not listen to the real symptom.

## Forbidden rationalizations

| Rationalization | Counter-argument |
|-----------------|-----------------|
| "It's obvious, no need to investigate" | Obviousness is the parent of the false fix. 2 minutes of investigation. |
| "I'll fix it and we'll see" | No. Hypothesis first, test, then fix. |
| "The bug is intermittent, it cannot be reproduced" | Make it reproducible (logs, seed, environment), or say so honestly. |

## If the process does not find a root cause

Say it explicitly: "I did not find the root cause. Here is what I ruled out: [...]. Next lead: [...]." Do not fabricate a random fix to save face.
