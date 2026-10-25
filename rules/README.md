# Rules per layer — rusty-agentic-pack-manager

Single source of truth for the rules of each DDD layer. **Shared**: the agents ([agents/](../agents/), [.vibe/agents/](../.vibe/agents/)) and the skills ([.agents/skills/](../.agents/skills/)) reference these files instead of duplicating instructions.

| Layer | File | Scope |
|---|---|---|
| Domain | [domain.md](domain.md) | `src/01_domain/` |
| Application | [application.md](application.md) | `src/02_application/` |
| Infrastructure | [infrastructure.md](infrastructure.md) | `src/03_infrastructure/` |
| Presentation | [presentation.md](presentation.md) | `src/04_presentation/` |
| Quality (cross-cutting) | [quality.md](quality.md) | `fmt`, `clippy`, tests, MSRV, global style |
| Tests (cross-cutting) | [testing.md](testing.md) | unit test conventions (co-located) & integration (`tests/`), decision threshold |

## Absolute cross-cutting rule

**Strictly inward** dependency flow — no outer layer may import an inner layer:

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

The UI never touches the domain directly: it goes through the application (DTOs, service traits).
