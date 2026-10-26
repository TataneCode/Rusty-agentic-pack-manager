---
name: tdd
description: "Use for any feature or bug fix, before writing implementation code: Red-Green-Refactor cycle with observed verification of RED then GREEN. Test first, code second, proof at every step."
license: MIT
---

# TDD — Test First

## The iron rule

**No implementation code before a failing test.** If you write functional code before the test, it is no longer TDD, it is "test after" — less reliable, more expensive.

## The Red-Green-Refactor cycle

### RED — Write a failing test

1. Write ONE test that describes the expected behavior.
2. It must be small, named for what it verifies (`should reject expired token`).
3. It must fail for the right reason (feature missing), not for a setup error.

### VERIFY RED — Watch it fail

Run the test. **Read the output.** Confirm it fails with the expected message. This is the proof that the test actually tests something.

### GREEN — Minimal code

Write the minimum code that makes the test pass. No elegance, no generalization, no neighboring feature.

### VERIFY GREEN — Watch it pass

Run the test. **Read the output.** It passes. The `verification-gate` applies here.

### REFACTOR — Clean up

Improve the code (names, structure) without changing behavior. The test suite must stay green after each small step.

### Repeat

One cycle per behavior. The next failing test starts the next cycle.

## A quality test

- Tests a behavior, not an implementation (do not couple the test to internal details).
- One concept per test; explicit name.
- Clear test data (nominal case, edge case, invalid input).
- Fast and independent (no ordering, no shared state).

## For a bug (retrospective TDD)

1. Write a test that **reproduces** the bug (it fails = RED).
2. Fix (GREEN).
3. Verify the test passes and the suite has no regression.

## Forbidden rationalizations

| Rationalization | Reality |
|-----------------|---------|
| "It's too small for a test" | The small thing is often what breaks the big thing. |
| "I'll write the test after, I know the code" | Testing after confirms your biases. |
| "The test is trivial" | A trivial test that fails then passes proves the fix. |
| "No time" | Time lost debugging untested code exceeds the time of the test. |

## Red flags — STOP and start over

- The code already exists and no test has seen it fail.
- You modify the test to make it pass instead of fixing the code.
- You write several features before making the first test pass.

## Verification checklist

- [ ] Every added behavior has its test written BEFORE the implementation.
- [ ] I saw the test fail (output read).
- [ ] I saw the test pass (output read).
- [ ] The full suite passes (output read).
- [ ] Refactoring keeps the suite green.

## When stuck

- Test refuses to go red → the behavior may already exist; check what the test really measures.
- Red test that won't go green → reduce the step: even more minimal code, or fall back to `systematic-debugging`.
- Too many tests failing in cascade → fix in dependency order, one by one.
