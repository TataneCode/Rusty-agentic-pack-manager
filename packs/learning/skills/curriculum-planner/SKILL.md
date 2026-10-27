---
name: curriculum-planner
description: "Plans and builds a coherent series of several lessons (e.g. \"7 days to learn Python\", an onboarding path, a semester split into sessions): clarifies the brief over several rounds, gets the full lesson list explicitly validated before producing anything, then builds the lessons one by one."
license: MIT
---

# Curriculum Planner — Planning a Series of Lessons

A series is several lessons the learner follows in order. The unit of work remains **one lesson at a time**. Three responsibilities at this level only: get a clear brief, get the user's **explicit validation of the full lesson list** before producing anything, and carry the series from start to finish without losing the thread.

## When to use

- The user asks for a sequence of several lessons: "7 days to learn Python", a 4-week onboarding path, a semester split into sessions, "turn this book into one lesson per chapter".

## When not to use

- For a single lesson, however big → use the lesson-design skill (`writing-plans` for the plan, `brainstorming` for the framing).

## Gate 1 — Clarify, over several rounds

A series request is almost always underdetermined. "7 days to learn Python" fixes the subject and the number of lessons, and leaves open everything that decides their content. Work in **2 or 3 rounds**, one at a time, each round narrower than the previous.

**Round 1 — where it lands.** The 2–3 things nothing else can decide without:

- **who the learner is** — absolute beginner, someone who codes in another language, a team of mixed levels;
- **what they want out of it** — a working script of their own, an exam to pass, a concept they can present.

Put the example in the option label: "complete beginner, never wrote a line of code" says what you mean by beginner.

**Round 2 — the form, adjusted to round 1's answers.** Now ask what round 1 made askable: the duration of a session, the pace, the teaching language, the degree of practice. **Build the options from what they just answered**: "complete beginner" becomes "short format: use a small tool / self-contained workshop / guided mini-project", and "already writes Java" becomes "cut to the Python/Java differences, or start from zero".

**Round 3 — the edges, and only if they are real.** Whether they have a book, a syllabus or existing material to build the series from; how many verifications they want (a quiz per lesson, a single one at the end, none). Skip this round when neither question would change the plan.

Rules for all rounds:

- **At most 3 main topics per round.** A round carrying six questions is a giant form in disguise.
- **Open each round by saying what you retain from the previous one.** "Since it's a complete beginner, 30 minutes per lesson: I compress each lesson onto an executable tool." The user must see their answer being used. That is why several rounds read as professional rather than slow.
- **Never ask what you already know.** What the request settled, or what you can reasonably default, is **stated as your decision, which the user can correct** rather than asked as a question. "Defaulting to English, 30 minutes per lesson, live" costs no round.

## Gate 2 — The confirmation gate

Before producing the slightest lesson, present the **complete list of lessons** of the series:

```markdown
# Series: [title]

## Retained brief
- Audience: [...]
- Objective: [...]
- Constraints: [...]

## Lesson list
| # | Title | Objective | Verification |
|---|-------|-----------|--------------|
| 1 | [...] | [what the learner will be able to do] | [exercise/quiz] |
| 2 | [...] | [...] | [...] |

## Order and dependencies
Lesson N builds on lesson M...

## Out of scope
[...]
```

**Wait for explicit validation.** The user validates, corrects, or reduces. No production before this validation — this is what distinguishes a series plan from a pile of lessons.

## Designing the series

- The backbone is a **small number of concepts** that come back, each at a growing level of complexity, abstraction or transfer. It is not a linear list of topics.
- Each lesson has: a learning objective, content, guided practice, a verification.
- Each lesson produces something verifiable (exercise, quiz, deliverable).
- The progression is readable: the learner must be able to say where they are and what comes next.

## The execution loop

For each lesson, in order:

1. **Design** the lesson (objective, content, practice, verification).
2. **Produce** the lesson (a concrete deliverable: sheet, workshop, exercise, quiz).
3. **Verify**: the lesson's verification is executed or solvable, and the result is recorded.
4. **Move to the next one**, briefly recalling the thread.

Long-context discipline:

- Do not rework a finished lesson unless explicitly asked.
- At each new lesson, recall in one line the previous lesson and what it produced.
- If a lesson fails or blocks, say so, mark it, and ask how to continue rather than improvising alone.

## To finish

When the last lesson is produced and verified:

- Summarize the series: lessons produced, verifications executed (commands + results if code), order followed.
- List what remains out of scope.
- Propose the logical next step (review, deepening, a lesson beyond the series).

## Verification before finishing

- [ ] The brief was clarified in rounds (not a giant form).
- [ ] The complete lesson list was **explicitly validated** before production.
- [ ] Each lesson has objective + content + practice + verification.
- [ ] Each lesson was produced and its verification executed/recorded.
- [ ] The thread stayed visible from one lesson to the next.
- [ ] Out of scope is stated at the end.
