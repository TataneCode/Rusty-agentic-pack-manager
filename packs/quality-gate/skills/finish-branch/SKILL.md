---
name: finish-branch
description: "Use when the implementation is done and the work must be integrated: verify the tests, determine the base branch, present the options (local merge or PR), execute the choice, clean up the workspace."
license: MIT
---

# Finish Branch — Finishing a branch

## When to use

Implementation finished, work ready to be integrated. Does not apply to work in progress.

## The process

### Step 1 — Verify the tests pass

Run the project's test suite (detect the command: `npm test`, `pytest`, `go test`, `cargo test`...). **Read the output.**

- All green → go to step 2.
- Red → back to work (`systematic-debugging` or `tdd`). Do not finish a branch that breaks.

### Step 2 — Diff review

Show the full diff (`git diff <base>...HEAD` or `git status` + `git diff`). Reread it like a reviewer (see `code-review`, self-review part). Fix what blocks.

### Step 3 — Determine the base branch

- Repository default branch: `main` or `master` (check: `git remote show origin` or the config).
- Explicit target branch (upstream feature branch): that one.
- In doubt, ask.

Verify the base is up to date (`git fetch`) and note any conflicts to expect.

### Step 4 — Present the options

Present to the user, do not decide for them:

| Option | When | Effect |
|--------|-------|-------|
| **1. Local merge** | Local feature branch, base = local main | Merge into the base, tests re-verified on the result |
| **2. Push + PR** | Shared repository, review required | Push the branch, open the PR, branch review tracking |
| **3. Leave as is** | Experimental work, not ready | Touch nothing, summarize the state |

### Step 5 — Execute the choice

**Option 1 — Local merge:**

```bash
git checkout <base>
git pull
# merge --no-ff to keep the branch history, or a plain merge per the repository convention
git merge <branch>
# re-verify the tests on the merged result
```

**Option 2 — Push + PR:**

```bash
git push -u origin <branch>
# open the PR (gh pr create or link) with: title, diff summary, executed verifications, review points
```

**Option 3 — Leave as is:** summarize the state in 3 lines (what, where, next step).

### Step 6 — Clean up the workspace

- If the work used a worktree: remove it (`git worktree remove <path>`).
- Delete dead local branches if merged.
- Verify no temporary file, debug log or instrumentation remains (`git status` clean).

## If the user asks to abandon the work

Delete nothing without explicit confirmation. Propose: `git checkout <base>` + branch kept, or `git branch -D <branch>` if truly discarded. Destroying uncommitted work is irreversible.

## Forbidden rationalizations

| Rationalization | Reality |
|-----------------|---------|
| "The tests passed before, no need to re-verify" | The merge changes the result. Re-verify (steps 1 and 5). |
| "The diff is big, no time to reread it" | It is precisely the big diff that deserves the reread. |
| "I merge directly, review will come after" | No. Review before merge, or explicit local merge. |

## Final checklist

- [ ] Test suite executed and green (output read).
- [ ] Diff reread, self-review done.
- [ ] Base branch identified and up to date.
- [ ] Option chosen by the user, executed.
- [ ] In case of merge: tests re-verified on the result.
- [ ] Workspace cleaned, `git status` clean.
