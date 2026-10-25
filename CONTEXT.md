# rusty-agentic-pack-manager — Project Map (CONTEXT.md)

**Point d'entrée unique.** Lis ce fichier en premier ; il te dit quoi lire ensuite — et quoi ne PAS lire. Règle d'or : avant de parcourir le code, consulte la map ci-dessous et limite ta lecture au périmètre de la tâche.

## En 30 secondes

TUI Rust pour gérer des **packs agentiques** : des ensembles de skills, agents et commands applicables à un CLI cible (Vibe CLI, Gemini CLI) et installables dans n'importe quel dépôt. Stack : `ratatui` + `crossterm` (TUI), `syntect` (coloration), `serde` + `toml` (manifestes). Édition 2021, **MSRV 1.88**. Aucun démon, aucun réseau : pur filesystem local. Le design, les patterns d'actions et l'outillage agentique viennent de [rusty-console](https://github.com/TataneCode/rusty-console).

## Architecture — DDD 4 couches, flux strictement inward

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Aucune couche externe ne peut importer une couche interne. L'UI ne voit jamais le domain : DTO + traits de service uniquement.

## Map de navigation — quoi lire, quand

| Tu veux… | Lis | Ne lis PAS |
|---|---|---|
| Comprendre le projet | Ce fichier + [README.md](README.md) | AGENTS.md en entier, tout src/ |
| Les touches / l'usage | [MANUAL.md](MANUAL.md) | — |
| Cadrer une demande | Skill `/challenge` | — |
| Planifier une feature | `/plan` + `rules/README.md` | tout src/ avant le cadrage |
| Le format pack / manifeste | `packs/tdd/pack.toml` + [MANUAL.md](MANUAL.md#format-dun-pack) | — |
| Coder — couche domain | `rules/domain.md` + `src/01_domain/pack/` | les autres couches |
| Coder — couche application | `rules/application.md` + `src/02_application/{pack,pack_view,explorer}/traits.rs` | `03_infrastructure` |
| Coder — couche infrastructure | `rules/infrastructure.md` + `src/03_infrastructure/` | `04_presentation` |
| Coder — UI | `rules/presentation.md` + `src/04_presentation/tui/` | `01_domain` directement |
| Écrire des tests | `rules/testing.md` + skill `rust-testing` | — |
| Livraison / qualité | `rules/quality.md` (fmt, clippy, tests) | — |
| Reviewer un diff / une PR | `/self-review` puis `/review` + `rules/` | — |
| L'historique des plans | `docs/plans/` | — |

## État du projet

**V1 en cours** — socle livré : split-screen packs/explorer, apply (`A`) + install (`i`), visionneuse (`v`) avec coloration syntaxique, création de dossier (`r`), aide contextuelle (Espace), 13 packs embarqués, packaging RPM/DEB/7z en CI. Plan : [docs/plans/init.md](docs/plans/init.md).

Hors périmètre v1 (prévu v2) : undeploy, rename (`m`), delete (`d`/`D`), packs 7z/tar, CLIs autres que vibe/gemini.
