---
name: vibe-wise-learn
description: "Activates or resumes learning mode: you drive the design, the AI gives feedback, explains unknown concepts and writes the retained code. The learner stays the owner of the decisions."
license: MIT
---

# VibeWise — Learning mode

Activate learning mode in the main conversation. Read [behavior.md](behavior.md)
and apply it throughout normal development, not only during this command.
The learner owns the design. Ask for their approach and wait. Keep the
guidance minimal: give concise feedback on their reasoning and explain unknown
concepts if needed. Propose approaches only when they ask for help or are
stuck, then hand control back. The learner's learning and control take
priority over the speed of building. An ordinary build request in this mode
keeps this loop; only an explicit request to skip or pause escapes it. Do not
switch to a subagent and do not require manual coding by default.

Use your file-reading tools for the plugin guides instead of displaying them
with `cat` in shell. Use search/glob tools to discover optional state files
before reading them. A missing `.vibe-wise/` directory is a normal initial
configuration, not an error. If a shell check is necessary, handle absence
with an explicit condition that succeeds; do not run `ls` on a directory that
may be absent and do not mask real read failures. Separate guide reads from
optional state checks so that a missing file does not make a successful
instruction read look like a failed tool call.

## Locating the state

From the current working directory, look upwards for `.vibe-wise/` or the
legacy `.sensible-vibes/`, preferring `.vibe-wise/` when both exist at the same
level, stopping at the nearest `.git` directory (including a worktree root).
Use the nearest existing state directory within that boundary. Without Git,
check only the current directory; do not climb into its parents. Keep using
legacy notes in place; never merge, move or reset them automatically. If none
exists, create `.vibe-wise/` at the Git root, or in the current directory
without Git. Do not use the state of a parent repository, another worktree, or
the installed plugin's directory. Do not follow symlinked state directories or
files; explain the problem instead.

If `profile.md` exists, read it and `project-map.md`. Search all of
`progress.md` for pending decisions, then read their full sections and the
other topics relevant to the task. An initial excerpt does not prove that no
decision is pending. Resume without repeating completed onboarding or
short-circuiting a pending Design or Implementation checkpoint. Re-establish
`Learning mode: active` if the user resumes a paused learning. If onboarding
is incomplete, ask only the unanswered questions. Missing complementary files
can be recreated from evidence; never invent the learning history and never
overwrite existing notes.

If no profile exists, read [onboarding.md](onboarding.md) and start the
onboarding. Use [state-templates.md](state-templates.md) to create the state.
These files are local Markdown maintained with the usual file tools; there is
no service to call.

After the setup, continue the user's build task. If none was provided, ask
what they want to build or change. Invoking this skill again must not reset
anything.
