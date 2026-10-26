# Reference — Plan format and validation checklist

Detail of the deliverable and final validation of `rust-architecture-planning` (steps 5 and 6). Loaded on demand.

## Deliverable format

```markdown
# Plan: <title>

## Goal
<one sentence>

## Architecture decisions
- D1: <decision> — <reason>
- D2: ...

## Steps
1. <atomic step> — files: <paths> — verification: <command>
2. ...
   (every step leaves the repo compilable and the tests green)

## Risks and mitigations
- <risk> → <mitigation>

## Out of scope
- <what is explicitly excluded>
```

- One step = one potential commit; layer order: domain → application → infrastructure → presentation → wiring.
- Every step cites its verification commands (`cargo check`, `cargo test <name>`, etc.).

## Final validation checklist

- The plan contains no implementation code (short illustrative snippets allowed for key APIs).
- Consistency: every mentioned file exists or is explicitly created; no step violates the dependency flow.
- Anticipated tests: every business-logic step has its associated test (see the `rust-testing` skill and `rules/testing.md`).
