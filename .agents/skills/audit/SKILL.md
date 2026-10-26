---
name: audit
description: Analyze an area of the produced code — rules compliance per layer, technical debt, DDD flow violations, quality. Produces a structured report. Use to audit an area, prepare a refactoring, or assess the state of a code zone. Strict read-only.
user-invocable: true
---

# Skill: Code audit (critical eye on the produced code)

Take the **Auditor** role: analyze the produced code, **never fix it**. Works from any entry point (Vibe Code, Vibe CLI).

## Input

- An area to audit: a folder (`src/01_domain/pack/`), a layer, a theme (error handling, tests, state/views/actions separation), or "the whole project".
- If no area is specified: audit the most recently modified layer (`git log --name-only -20`).

## Procedure

1. **Frame**: read the project map and the rules index. The audit judges against the rules — de facto rules — never against personal preferences.
2. **Delimit**: list the files of the scope (do not read the whole project; follow the map).
3. **Examine per layer** (blocking rules first):
   - **DDD flow**: no import of an inner layer from an outer one (`grep` the `use crate::` in the scope).
   - **Domain**: zero external crates (thiserror tolerated), invariants in types, dedicated domain errors, no I/O.
   - **Application**: mockable traits, stateless services, dedicated DTOs, application errors.
   - **Infrastructure**: foreign types confined, infrastructure error → application error.
   - **Presentation**: state/views/actions separation (app.rs / split_view+viewer / router.rs per `rules/presentation.md`), pure views, keys in the key mapping module, errors mapped to popup messages.
   - **Cross-cutting**: thiserror error chains, no `unwrap()`/`expect()` outside tests on error paths, tests present for business logic.
4. **Assess the debt**: duplicated code, abnormally large files, accumulated `#[allow]`, TODOs, missing tests on recent logic.
5. **Verify read-only** (if useful): `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` — observations, not fixes.

## Report format

```markdown
# Audit: <area>

## Summary
<3-5 lines: overall state, DDD compliance, main debt>

## Findings
| # | Severity | Location | Finding | Suggestion |
|---|---|---|---|---|
| 1 | 🔴/🟡/🟢 | `file:line` | <what> | <what to do, without code> |

- 🔴 blocker (violates the rules, bug, test gap on business logic)
- 🟡 important (debt, bypassed pattern, readability)
- 🟢 minor (style, naming)

## Observed good practices
<what is well done — an audit is not just a list of defects>

## Leads (out of delivery)
<possible refactors or improvements, to be planned via /plan if retained>
```

## Limits

- **Strict read-only**: no file modified, no commit, no "by the way" fix.
- A finding without `file:line` and without a concrete suggestion is not a finding.
- Do not suggest style contrary to the repo conventions (the rules are the authority).
- The report is a text deliverable; its publication (issue, PR comment) goes through the confirmation gate.

## Forbidden anti-patterns

- An "at a glance" audit without reading the scope's files.
- Confusing debt and violation: debt gets planned, a violation is flagged 🔴.
- Proposing code in the report (descriptive suggestions only).
- Auditing the whole project when one area suffices — the scope gets delimited.
