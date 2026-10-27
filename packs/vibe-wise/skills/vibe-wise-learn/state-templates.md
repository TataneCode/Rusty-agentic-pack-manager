# Local State Templates

Create only these three files in the `.vibe-wise/` of the chosen project, or
its legacy `.sensible-vibes/` when resuming. Use the directory selected by
SKILL.md. Replace the bracketed values with real evidence or "Not specified".
Keep the two status lines unformatted and near the top. Do not replace an
existing state with a fresh template.

## profile.md

```markdown
# Learner Profile

Learning mode: active
Onboarding: complete

## Project
Situation: [New / Existing / Known]
Building: [project objective]
Codebase familiarity: [answer]
Learning scope: [entire system / parts we touch / mixed]

## Experience
Overall programming: [Beginner / Intermediate / Advanced, or Not specified]
Stack familiarity: [levels by technology if given; otherwise Not specified]

## Goals
Primary: [answer]
Capability goal: [optional answer]

## Preferences
Checkpoint frequency: Normal
Question style: Open-ended
Implementation style: AI writes code

## Strong Concepts
No demonstrated understanding recorded yet.

## Developing Concepts
None recorded yet.

## Revisit
None recorded yet.
```

## progress.md

```markdown
# Learning Progress

No learning events recorded yet.
```

Over the course of learning, add a `## Topic` with concise bullets under
Introduced, Demonstrated understanding and Needs reinforcement. Record
evidence of reasoning, not quotes of a whole exchange. Product preferences
establish needs; they are not evidence of engineering understanding. Keep the
learner's proposed reasoning distinct from concepts explained by the AI.
Consolidate repeated entries. Keep each topic readable independently so it can
be loaded without the whole file. While waiting for a checkpoint, keep a short
section titled exactly `## Pending decision`, outside a topic, with the proposed
approach and the expected answer. Include the decision name and the checkpoint
step: awaiting reasoning, choice confirmation, or implementation approval.
Remove it once resolved: record the result in the map as chosen or implemented,
mark approved additions as AI's, and any reasoning evidence in its topic. Do not
move the pending text into a topic; update the entries the result replaces.
Record confirmed choices in the map without claiming they are implemented. Keep
any proposed coding scope explicit. The confirmation covers only the presented
proposal. Do not add fields, behaviors, rejected alternatives or unmentioned
reasons to the chosen design. Mark unresolved details as unknown and the AI's
suggestions as proposed; never attribute them to the learner.

## project-map.md

```markdown
# Project Map

## Objective
[What this software does.]

## Needs
[The user's needs and constraints. They do not automatically decide technical choices.]

## Components
[Components, responsibilities and associated file paths.]

## Main Flow
[Compact text diagram with labeled arrows. Mark the unknowns and distinguish
proposed, chosen and implemented components. Reflect the learner's model refined
together, or the verified existing code; do not fill missing relationships with
assumed designs.]

## Data and Trust Boundaries
[Storage, ownership, auth, external services; unknown if not verified.]

## Build and Deployment
[Commands and configuration paths verified in the repository.]

## Unknowns
[Unresolved technical choices and what needs inspection, including stack,
storage location/model, data structures, interfaces and deployment choices.]
```
