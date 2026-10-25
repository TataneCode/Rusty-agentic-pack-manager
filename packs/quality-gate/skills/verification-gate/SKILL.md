---
name: verification-gate
description: "Use before claiming that work is finished, fixed or passing, before any commit or merge: require running the verification command and observing the output. No success claim without executed proof."
license: MIT
---

# Verification Gate — Verification gate

## The iron rule

**No success claim without executed proof.** "It should work", "the fix is in place", "the tests pass" — these are not proofs. The only proof is a command executed in this session whose output you have read.

## The gate

Before saying something is finished, fixed or verified, apply the gate function:

1. **Which command proves this claim?** (test, lint, build, request, script)
2. **Run it.**
3. **Read the output.** Partial success, warnings, unexpected output = not a success.
4. **Cite the evidence** in your answer: command + observed result.

If no command can prove the claim, say so: "I could not verify X, because Y." Do not pretend.

## What counts as proof

- Test output: `X passed, 0 failed` (read, not assumed).
- Build/lint output: code 0, no errors.
- Executed request: observed response (HTTP code, content).
- Produced file: reread after writing.
- Bug reproduction: the reproduction test fails BEFORE the fix, passes AFTER (red-green proof).

What does NOT count: "the code is correct", "I am confident", "it worked last time", a test you did not run.

## Rationalization table (prohibitions)

| Rationalization | Reality |
|-----------------|---------|
| "It's a small change, no need to verify" | Small changes break too. The gate applies to everything. |
| "I verified in a previous session" | This session, this proof. Re-run. |
| "The tests are surely green" | "Surely" is not a proof. Execute. |
| "CI will handle it" | CI verifies after the fact; you verify before claiming. |

## Red flags — STOP

- You are about to write "done", "fixed", "it works" without having run the verification in this turn.
- The verification command failed and you are considering saying it is fine anyway.
- You cite evidence you do not have in front of you.

## When the gate applies

- End of task, end of bugfix, end of feature.
- Before any commit, push, merge, PR.
- Before answering "it's fine" to the user.
- Exception: the task is purely conversational (explaining a concept, drafting text without execution) — no gate, but claim no execution either.

## Integration with the other skills

- `tdd`: the gate is the "verify RED" and the "verify GREEN".
- `systematic-debugging`: phase 4 (fix) only ends once the gate is passed.
- `finish-branch`: step 1 = run the test suite, read the output.
