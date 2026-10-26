---
name: rust-code-review
description: Review a GitHub pull request of this project — systematic per-DDD-layer examination, quality control, actionable verdict. Use this skill for any PR review (requested, assigned, or CI monitoring).
---

# Skill: Rust code review (pull requests)

## Systematic procedure — 6 steps

### 1. Collect the context
```bash
gh pr view <n> --repo TataneCode/Rusty-agentic-pack-manager
gh pr diff <n>
```
- Read the title, the description and the linked issue.
- Check the diff size: a PR with > ~400 lines of logic is a signal of insufficient splitting (note it as a comment, not systematically blocking).

### 2. Verify the scope
- Does the diff do exactly what the title/description announces? Any unrelated change = ask to extract it.
- No generated files nor needlessly churned `Cargo.lock` (except a justified dependency change).

### 3. Examine per layer (blocking rules)
- **Domain (`01_domain`)**: no external crates; invariants validated in the types; `DomainError` for every construction failure.
- **Application (`02_application`)**: traits `#[automock]`; stateless services; dedicated DTO/mappers; no exposed domain entity.
- **Infrastructure (`03_infrastructure`)**: `toml` and raw `std::fs` types confined here; mappers to the domain; errors mapped to `ApplicationError` with the failing path.
- **Presentation (`04_presentation`)**: state/views/actions separation respected (app.rs / split_view+viewer / router.rs per `rules/presentation.md`); no business logic in the UI; `AppAction`/`KeyContext` extended cleanly; errors caught and shown as popups.
- **Cross-cutting**: thiserror error chain respected; no new `unwrap()`/`expect()` on an error path; `main.rs` wiring consistent.

### 4. Control quality and tests
- New business logic → tests present and meaningful (no test testing the mock).
- Bugfix → regression test present (see the `rust-bugfix` skill).
- If CI is running: check its verdict; on failure, find the root cause in the logs before any comment.
- `cargo clippy -- -D warnings` and `cargo fmt` must be clean (verifiable locally on the branch if needed).

### 5. Write the review
- Structure: **Verdict** (approve / request changes / comment) up front, then findings by severity:
  - 🔴 **Blocker** — violates the architecture, breaks CI, bug, security
  - 🟡 **Important** — bypassed pattern, missing test, readability
  - 🟢 **Minor / nit** — style, naming (non-blocking)
- Every finding: `file:line`, explanation of the problem, concrete suggestion. No "this is ugly" without an alternative.
- Explicitly acknowledge good decisions (a review is not just a list of defects).

### 6. Publish and follow up
- Publish via `gh pr review` with a structured body (temp file + `--body-file`).
- After the author's responses: re-verify only the fixed points, not the whole PR.

## Forbidden anti-patterns

- A superficial "LGTM" review on a diff containing business logic.
- Findings without location or suggestion.
- Requesting personal style contrary to the repo conventions (they are the authority, via `AGENTS.md` and the skills).
- Approving a PR that violates the DDD dependency flow "because it is small".
