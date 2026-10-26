---
name: prd
description: "Write a PRD (Product Requirements Document) optimized for an agent: mandatory clarification with the user, limited quick research, structured writing, self-assessment. Use to frame a new feature or product."
license: MIT
---

# PRD — Product Requirements Document

## Phase 0 — Clarification (mandatory)

Before writing a single line of PRD, ask the user these questions:

1. **Target users**: who will use it? (developers, end users, admins...)
2. **Core problem**: what pain does it solve? Numbers if available.
3. **Success criteria**: how do we measure success?
4. **Constraints**: technical, budget, deadline.
5. **Context**: greenfield or integration into an existing system?

**Wait for the answer before continuing.** If the user says "skip" or gives the description inline, extract what you can and note the assumptions.

## Phase 1 — Quick research (max 2 searches)

Only if the domain is unknown:

- 1 search for domain/market context.
- 1 search for possible technical patterns.

No more. 60 seconds max for this phase. No deep research.

## Phase 2 — Writing the PRD

```markdown
# PRD: [Name]

## 1. Vision
One sentence: what the product/feature accomplishes.

## 2. Target users
- [Profile 1]: need, usage context.

## 3. Problem
What pain, for whom, today. Frequency and impact.

## 4. Goals and success criteria
- [Goal 1] → measurable: [metric]

## 5. Features

### Must have (MVP)
- [Feature]: description, user, acceptance criterion.

### Should have
- ...

### Won't have (out of scope)
- ...

## 6. Constraints and assumptions
- Constraints: [technical, budget, deadline]
- Assumptions: [to validate]

## 7. Open questions
- [Question] → who decides, when.

## 8. Next steps
1. [Concrete step, e.g.: validate this PRD, then slice into a plan (`writing-plans`)]
```

## Phase 2.5 — Adversarial review (recommended)

Reread your own PRD with a hostile eye:

- Is every "must have" truly indispensable to the MVP?
- Is every success criterion measurable, not fuzzy?
- Is there a "just in case" feature to reclassify as "won't have"?
- Are the assumptions testable?

Fix what the review reveals. Mark the changes.

## Phase 3 — Self-assessment

Score the PRD out of 10 with this grid, and justify every score in one line:

| Criterion | Score /10 |
|---------|----------|
| Problem clarity | |
| Well-defined users | |
| Measurable success criteria | |
| Realistic MVP scope | |
| Open questions identified | |

If a score is < 7, fix before presenting. If all are ≥ 7, present.

## Phase 4 — Save

Store the PRD in the repository (proposal: `docs/prd/<date>-<topic>.md`) and summarize in 5 lines: vision, MVP, key metric, main constraint, next step.

## Rules

- The PRD describes the **what** and the **why**, never the **how** (no technical implementation choices).
- A success criterion is measurable. "Users love it" is not a criterion.
- The MVP is the smallest set that proves value. Cut without mercy.
