---
name: self-review
description: Reread your own diff before requesting an external review — rules checklist per layer, cross-layer import verification, local quality gate. Use systematically after /implement and before /review or a human review request. The noise filter that saves the most context.
user-invocable: true
---

# Skill: Self-review (reread your own work)

**Developer** role: verify your own diff **before** submitting it. Works from any entry point (Vibe Code, Vibe CLI).

## Goal

Filter the noise before an external review: most review findings are detectable alone in 2 minutes. A systematic self-review reduces the context consumed by `/review` and the number of back-and-forths.

## Procedure

1. **List the diff**: `git status` + `git diff` (unstaged) or `git diff main...HEAD` (whole branch). Read every modified file **in full**.
2. **Verify the scope**: does the diff do exactly what the task asked? Any unrelated change is removed or flagged — never "by the way".
3. **Checklist per layer** (blocking rules):
   - DDD flow: `grep -rn "use crate::"` on the modified files — no import of an inner layer from an outer one.
   - Domain: zero external crates, invariants in types, no I/O.
   - Application: stateless services, dedicated DTOs, application error type, no exposed domain entity.
   - Infrastructure: foreign types confined, infrastructure error → application error.
   - Presentation: state/views/actions separation (app.rs / split_view+viewer / router.rs per `rules/presentation.md`), pure views, keys in the key mapping module, errors mapped to popup messages.
   - Cross-cutting: no `unwrap()`/`expect()` added outside tests on an error path; no `#[allow]` added to bypass clippy (rename/refactor instead).
4. **Verify the tests**: new logic = co-located unit test; new user-facing behavior = integration scenario; bugfix = regression test that failed before the fix.
5. **Run the local quality gate** (in order):
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   cargo test
   ```
   Any red → fix before continuing. **Never claim "it works" without executed proof.**
6. **Write the summary**: what was verified, what stays open (e.g. integration test to add, point to discuss in review).

## When to stop

- The gate is green and the checklist is ticked → the diff is ready for `/review` or a human review.
- A checklist item reveals a structural doubt → fix it, or flag it explicitly for the review (do not hide it).

## Forbidden anti-patterns

- A self-review "from memory" without rereading the diff.
- `cargo test` green but `clippy` not passed (the gate is a whole).
- Fixing a review finding with a "quick commit" without rerunning the full self-review.
- Requesting an external review for a diff that has not passed the local gate.
