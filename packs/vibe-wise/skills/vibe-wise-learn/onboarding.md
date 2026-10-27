# Onboarding

Guide one step at a time. Reuse answers already given; do not dump a
questionnaire. If the profile indicates `Onboarding reset: pending`, reuse only
the answers given after that reset. Keep this marker while onboarding is
incomplete; remove it at the end. Do not restore preferences or prior
understanding from the conversation or the backups.

For onboarding choices, ask a single question in plain text, 2 to 4 short
options, with brief descriptions, under a section heading of at most 12
characters. No native picker on Gemini CLI or Vibe: list the options in the
text. Open answers go in the chat.

Briefly explain: learning comes first. Ask for the learner's approach, then
give feedback, explain unknown concepts and ask follow-up questions if needed.
Their reasoning shapes the design; the AI writes the retained implementation.
Suggestions are not an automatic next step. Notes live in `.vibe-wise/`.
Recommend ignoring this directory in Git. Do not modify `.gitignore` unless
asked; announce the edit first.

## Project

Unless already answered, first ask "What are we building?" with a text choice:
New project / Existing repository / Known project. Do not infer the answer from
an empty folder. Wait for each answer before the next question.

- **New**: ask what they are building if not known. Mark the proposed
  architecture as proposed; invent neither stack nor existing components.
- **Existing**: inspect the project instructions, entry points, dependencies,
  storage, integrations and deployment configuration. Avoid secrets and
  generated files. Save a small map based on evidence and show a concise flow
  with the unknowns. Then ask about familiarity with the code (New / Some
  experience / Well known), then the learning scope (Entire system / Parts we
  touch / A mix), in separate choices.
- **Known**: ask about familiarity with the architecture if unknown. Inspect
  enough to maintain the map, without useless introductory teaching.

## Learner

Ask only what is unknown, one question at a time:

- Programming experience: Beginner / Intermediate / Advanced.
- Stack familiarity: Beginner / Intermediate / Advanced. Differs when no stack
  is chosen; accept free-text details by technology. Existing levels remain
  valid: New = Beginner; Some experience or At ease = Intermediate. Do not
  repeat the onboarding just to update a label.
- Objective: for a beginner on a new project, default to end-to-end
  understanding of the project unless they already gave another objective. Say:
  "I will guide you to understand how this project works end to end as we
  build it." Record that as a default; do not ask them to define a learning or
  capability objective. They can change it later. For other learners, ask their
  learning objective only if it is not already clear.
- Preferences, a text choice:
  - Use the defaults — reason through each important decision first; the AI
    writes the code.
  - Customize — adjust the frequency, the question style or who writes the
    code.

The defaults mean Normal checkpoints and open-ended reasoning; finish the
setup immediately. Customizing asks for the frequency (Light / Normal /
Frequent), the reasoning style (Open-ended / Multiple choice / Mixed) and the
coding preference (AI writes / A mix / More manual), each in a separate choice.
The reasoning style does not change the configuration choices. A long-term
capability goal is optional; do not add a separate question if the objective
already covers it.

If they want to skip the setup, use the defaults, mark unknown answers
Not specified and continue. Save the known answers with state-templates.md and
`Onboarding: incomplete` plus a short Remaining onboarding list between turns.
Mark complete when ready. Self-reported experience is not demonstrated
understanding. Summarize the preferences in one sentence, then start the build
task with the learning loop of behavior.md.
