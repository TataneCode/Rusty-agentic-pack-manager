# Agent Architecture — rusty-agentic-pack-manager

Tree usable by any AI agent (Mistral Vibe, or other), without depending on a proprietary ecosystem.

```
CONTEXT.md            project map: general context, navigation (what to read, when), project status
AGENTS.md             startup router: confirmation gate + routing table + sources of truth
rules/                per-DDD-layer rules, shared between agents and skills (single source)
  README.md                   rules index + dependency flow
  domain.md                   src/01_domain/
  application.md              src/02_application/
  infrastructure.md           src/03_infrastructure/
  presentation.md             src/04_presentation/
  quality.md                  cross-cutting: fmt, clippy, tests, MSRV, style
  testing.md                  cross-cutting: UT & IT conventions, decision threshold
.agents/skills/       reusable procedures (Agent Skills standard, discovered by Vibe Code and Vibe CLI)
  challenge/                  user skill /challenge (Challenger role, request framing)
  plan/                       user skill /plan (Architect role, plan approval mandatory)
  implement/                  user skill /implement (Developer role, commit/push protection)
  self-review/                user skill /self-review (reread your diff before external review)
  review/                     user skill /review (Reviewer role, GitHub PR or branch diff)
  audit/                      user skill /audit (Auditor role, produced code analysis)
  config-update/              user skill /update-config (evolve this configuration)
  rust-architecture-planning/   design and plans (+ references/plan-format.md)
  rust-feature-ddd/             layer-by-layer implementation (+ references/layer-checklists.md)
  rust-refactoring/             behavior-preserving refactoring
  rust-testing/                 test conventions (summary of rules/testing.md)
  rust-quality-gate/            quality control before delivery
  rust-bugfix/                  bug fixing
  rust-dependency/              Cargo dependency management
  rust-code-review/             pull request review
  layer-rules/                  per-layer rules index (rules/)
  confirmation-gate/            mandatory confirmation gate before any write
agents/               role definitions orchestrating the skills (agnostic)
  challenger.md               challenges the request before planning, never a plan nor code
  architect.md                produces plans, never code
  developer.md                exploits plans, produces and verifies code
  reviewer.md                 reviews GitHub PRs
  auditor.md                  analyzes the produced code, read-only, never fixes
.vibe/agents/         Vibe CLI agents (TOML): challenger.toml, architect.toml, developer.toml, reviewer.toml, auditor.toml
.vibe/prompts/        Vibe CLI agent system prompts
.vibe/config.toml     built-in modes disabled (only ask + project agents remain)
AGENT_ARCHITECTURE.md (this file) — how everything fits together
```

## Orchestration

| Phase | Agent | Invoked skills | Deliverable |
|---|---|---|---|
| Framing | **Challenger** | `challenge` | Restated request, blocking ambiguities, minimal scope |
| Planning | **Architect** | `rust-architecture-planning`, `rust-dependency` | Plan (goal, decisions, verifiable steps, risks, out of scope) |
| Implementation | **Developer** | `rust-feature-ddd` / `rust-refactoring` / `rust-bugfix`, `rust-testing`, `rust-dependency`, `rust-quality-gate`, `self-review` | Code + tests + green gate + self-review |
| Review | **Reviewer** | `rust-code-review`, `rust-quality-gate` | Structured review on the PR |
| Analysis | **Auditor** | `audit`, `layer-rules` | Audit report (findings 🔴/🟡/🟢, debt, leads) |

Nominal flow: the user expresses a need → the **Challenger** frames the request → the **Architect** produces a plan → the plan is handed to the **Developer** who implements, passes the quality gate and the self-review → the opened PR is reviewed by the **Reviewer**. The **Auditor** intervenes read-only to analyze a code zone. Each agent is independent and can also be invoked alone.

The user entry points are the user-invocable skills (slash-commands):
- `/challenge` — Challenger, read-only; restates, detects ambiguities and scope creep, proposes the minimal scope; use **before** `/plan` on a non-trivial request.
- `/plan` — Architect, read-only on the sources, all tools accessible; **explicit plan approval mandatory** before implementation; if approved, invokes `implement`.
- `/implement` — Developer, all tools accessible; **never commit nor push without user agreement**; `cargo build/test/fmt/clippy` gate + mandatory self-review before external delivery.
- `/self-review` — Developer, rereads their own diff (rules checklist, cross-layer imports, local gate) before `/review`.
- `/review` — Reviewer, strict read-only; in Vibe Code, GitHub PR review; in CLI, branch diff (current vs `main` by default).
- `/audit` — Auditor, strict read-only; analyzes a zone of the produced code, structured report.
- `/update-config` — evolve this configuration (skills, rules, agents, commands) while keeping consistency.

## Contract between agents

- The Challenger frames the request before the Architect plans; the Architect's plan is the only input format of the Developer for planned work — every step must be actionable without going back to the Architect.
- The Developer flags any deviation from the plan instead of executing it silently, and self-reviews their diff before any external review.
- The Reviewer and the Auditor judge against the skills and the layer rules (`rules/` — de facto rules), not against personal preferences; their verdicts/findings are always argued (`file:line` + suggestion).

## Skill format

Every skill lives in `.agents/skills/<name>/SKILL.md` with a YAML frontmatter (`name`, `description` — which serves as the invocation description), a systematic step-by-step procedure, and a forbidden anti-patterns section. Verbose details live in the skill's `references/`, loaded on demand (progressive disclosure).

The layer rules are **shared** in `rules/`: agents and skills reference them instead of duplicating instructions. The `layer-rules` skill serves as the index.

> Note: unlike rusty-console (from which the agent tooling was inherited), this project does **not** use a per-panel presenter/view/actions file triad. The three concerns are separated by location instead: display state in `tui/app.rs`, pure views in `tui/split_view.rs` and `tui/viewer/view.rs`, action handlers in `tui/router.rs`. The normative rule lives in `rules/presentation.md`.

### Invocation from runtimes

- **Vibe Code (sandbox)** and **Vibe CLI** natively discover the skills in `.agents/skills/` (Agent Skills standard, trusted folder). The frontmatter `user-invocable: true` exposes a skill as a slash-command (`/<name>`).
- **Vibe CLI** loads the agents from `.vibe/agents/*.toml` (selection: `vibe --agent challenger|architect|developer|reviewer|auditor`, or Shift+Tab interactively). The corresponding system prompts live in `.vibe/prompts/`. `.vibe/config.toml` disables the built-in modes (`plan`, `accept-edits`, `auto-approve`, `smart-approve`, `lean`): only `ask` and the project agents remain, with `ask` as default.
- Any other runtime can consume the agnostic Markdown files (`agents/`, `rules/`, `.agents/skills/`) without depending on a proprietary ecosystem.
