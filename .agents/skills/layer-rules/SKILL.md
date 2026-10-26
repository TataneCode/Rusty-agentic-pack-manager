---
name: layer-rules
description: Per-layer DDD rules for this repository (domain, application, infrastructure, presentation) + cross-cutting quality rules. Use this skill as soon as a task touches code in a layer, to load the applicable placement and style rules.
---

# Skill: Rules per layer (shared)

Every procedure that touches a layer's code must respect the corresponding rule files:

These files live at the repository root, in `rules/` (load them with `read_file`):

- `rules/domain.md` — `src/01_domain/`
- `rules/application.md` — `src/02_application/`
- `rules/infrastructure.md` — `src/03_infrastructure/`
- `rules/presentation.md` — `src/04_presentation/`
- `rules/quality.md` — verification commands and global style

## Quick reminder — dependency flow

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

No outer layer may import an inner layer. Before delivering, check the imports of every modified file.
