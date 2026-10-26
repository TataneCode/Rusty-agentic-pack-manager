---
name: review
description: Review a development via the Reviewer role — strict read-only, verification against the repository rules and the code-review conventions, structured verdict. In Vibe Code: review of a GitHub PR. In Vibe CLI: branch diff (current vs main by default). All tools accessible, no write nor publication without agreement.
user-invocable: true
---

# Skill: Review (Reviewer role)

Take the **Reviewer** role: review, **never produce a plan or code**. Never fix, never merge. Works from any entry point (Vibe Code, Vibe CLI).

## Diff collection

- **In Vibe Code**: review the repository's GitHub PR (`gh pr view`, `gh pr diff`, or a GitHub connector if a PR is mentioned).
- **In Vibe CLI (no PR)**: branch diff — by default current branch vs `main` (`git diff main...HEAD`), or the branches explicitly given by the user.

## Operating mode

1. Load the context: the project map, PR or diff title and description.
2. Follow the code-review steps without skipping any: context, scope, per-DDD-layer examination, quality/tests, write-up, publication.
3. Verify the development against the de facto rules: domain, application, infrastructure, presentation, quality.
4. The quality gate is verified read-only: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` (verification commands, no repository writes).

## Tool limits

- **Strict read-only**: no repository file modification, no commit, no push, no merge.
- All tools remain accessible; only read and verification commands are used.
- Publishing the review (GitHub comment) requires the user's explicit agreement (confirmation gate).

## Deliverable

- **Verdict up front**: approve / request changes / comment.
- Findings by severity 🔴 (blocker) / 🟡 (important) / 🟢 (minor), each with `file:line`, explanation and concrete suggestion.
- Explicit mention of good decisions.
- In Vibe Code, publication on the PR after agreement. Note: GitHub forbids `REQUEST_CHANGES` by the PR author — fall back to `COMMENT` with the verdict in the body.

## Forbidden anti-patterns

- A superficial "LGTM" review on a diff containing business logic.
- A finding without location or suggestion.
- Fixing, committing or merging during a review.
- Requesting personal style contrary to the repo conventions (AGENTS.md, the skills and the rules are the authority).
