---
name: implement
description: Implement a feature or bugfix via the Developer role, from an approved Architect plan — all tools accessible, never commit or push without the user's explicit agreement, validation via build / test / fmt / clippy, PR check monitoring in Vibe Code.
user-invocable: true
---

# Skill: Implement (Developer role)

Take the **Developer** role: implement and verify, **never produce a plan**. Works from any entry point (Vibe Code, Vibe CLI).

## Input

- An **approved** plan from the `plan` skill, or a direct instruction precise enough.
- If no plan is provided and the task is not trivial: ask to go through the `plan` skill rather than improvising.

## Operating mode

1. Load the context: the project map, the plan, and the rules of the layers touched.
2. Apply the skill matching the work type: feature (layer by layer, default), bugfix (reported bug or red test), refactoring (identical behavior) — plus testing conventions for any new logic.
3. Execute the plan steps in order; one step = one potential commit, the repo stays green at every step. Any necessary deviation from the plan is flagged, never done silently.
4. Run the **full quality gate** before declaring done: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy -- -D warnings`.
5. **Mandatory self-review** before any external delivery (check the rules checklist, cross-layer imports, reread the local gate on the diff) — then `/review` if an external review is requested.

## Protection rules (non-negotiable)

- **No `git commit` nor `git push` without the user's explicit prior agreement**, whatever the entry point. The repository stays in a work state until that agreement.
- Every write stays covered by the confirmation gate: scope presented, agreement received, then execution as presented.

## In Vibe Code — PR monitoring

After opening a PR (with user agreement):
1. Monitor the PR checks (GitHub Actions: build, test, clippy, fmt).
2. On failure, identify the root cause in the logs, fix locally, rerun the quality gate.
3. A new push to deliver the fix requires a new agreement.

## Final report

Steps done, files touched, gate result (executed commands + verdict), deviations from the plan with the reason.

## Forbidden anti-patterns

- Committing or pushing without explicit agreement.
- Silently modifying the plan's scope.
- Bypassing a rule from the skills or the repository rules, even to go faster.
- Declaring done without an observed green gate (commands executed and read).
