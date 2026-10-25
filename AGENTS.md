# AGENTS.md — rusty-agentic-pack-manager

Routeur de démarrage pour tout agent IA. **Ne lis pas tout le projet** : commence par [CONTEXT.md](CONTEXT.md) (project map), consulte cette table, puis lis uniquement les fichiers du périmètre de la tâche.

## Confirmation Gate (obligatoire)

Avant **toute** action d'écriture sur ce dépôt (fichier, branche, commit, push, PR, merge), l'agent DOIT présenter un plan concis et attendre l'accord explicite de l'utilisateur. Les opérations purement en lecture (read, grep, `cargo test`/`clippy`/`fmt --check`/`build`) sont exemptées. Le silence ou une réponse ambiguë signifie « ne pas continuer ». Procédure complète : [.agents/skills/confirmation-gate/SKILL.md](.agents/skills/confirmation-gate/SKILL.md).

## En 15 secondes

TUI Rust de gestion de **packs agentiques** (skills, agents, commands) pour CLIs IA (Vibe CLI, Gemini CLI) : lister, visualiser, installer dans un dépôt. DDD 4 couches, flux strictement inward :

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Stack : ratatui + crossterm, syntect (coloration), serde + toml (manifestes), thiserror. Application **synchrone** (std::fs, pas de tokio). Édition 2021, MSRV 1.88. Commandes et état du projet : [CONTEXT.md](CONTEXT.md).

## Routage — quelle commande pour quelle tâche

| Commande (skill `/…`) | Rôle | Quand l'utiliser |
|---|---|---|
| `/challenge` | Challenger | **Avant tout `/plan`** : reformuler la demande, détecter ambiguïtés et scope creep, proposer le périmètre minimal |
| `/plan` | Architecte | Cadrer une feature, un bugfix ou un refactor avant d'écrire du code (plan approuvé obligatoirement avant implémentation) |
| `/implement` | Développeur | Implémenter à partir d'un plan approuvé (jamais de commit/push sans accord) |
| `/self-review` | Développeur | Relire son propre diff (checklist `rules/`, imports inter-couches, gate local) **avant** de demander une review |
| `/review` | Reviewer | Revue de PR GitHub (Vibe Code) ou de diff de branche (Vibe CLI), verdict structuré |
| `/audit` | Auditeur | Analyser une zone du code produit : conformité `rules/`, dette, violations DDD — lecture seule |
| `/update-config` | — | Faire évoluer cette configuration (skills, rules, agents, commandes) |

Skills métier invoqués par les rôles ci-dessus : `rust-architecture-planning`, `rust-feature-ddd`, `rust-bugfix`, `rust-refactoring`, `rust-testing`, `rust-quality-gate`, `rust-dependency`, `rust-code-review`, `layer-rules` — voir [.agents/skills/](.agents/skills/).

## Sources de vérité (une seule par sujet)

| Sujet | Fichier |
|---|---|
| Contexte projet, map de navigation | [CONTEXT.md](CONTEXT.md) |
| Règles par couche DDD | [rules/](rules/) (`domain`, `application`, `infrastructure`, `presentation`) |
| Qualité transverse (commandes, style) | [rules/quality.md](rules/quality.md) |
| Conventions UT & IT | [rules/testing.md](rules/testing.md) + [docs/TESTING.md](docs/TESTING.md) (matrice des scénarios) |
| Format pack / manifeste | [MANUAL.md](MANUAL.md) + `packs/tdd/pack.toml` (exemple) |
| Packs embarqués | [packs/](packs/) |
| Procédures agents/skills | [.agents/skills/](.agents/skills/) + [AGENT_ARCHITECTURE.md](AGENT_ARCHITECTURE.md) |
| Rôles d'agents (Markdown agnostique) | [agents/](agents/) |
| Wiring Vibe CLI | [.vibe/](.vibe/) (`agents/`, `prompts/`, `config.toml`) |

## Conventions incontournables

- **Lire un fichier entièrement avant de le modifier** ; diff minimal ; matcher le style existant.
- **Erreurs** : `thiserror` par couche — `DomainError` → `ApplicationError` (`#[from]`) pour l'UI ; `anyhow` réservé à `main.rs`. Jamais de `String` d'erreur dans les couches.
- **Tests** : toute logique nouvelle a son test ; bugfix = test de régression d'abord (rouge) puis correction. Voir [rules/testing.md](rules/testing.md).
- **Gate avant livraison** : `cargo fmt` → `cargo clippy -- -D warnings` → `cargo test`. Voir [rules/quality.md](rules/quality.md).
- **Jamais de commit, push, PR ou merge sans accord explicite de l'utilisateur** (voir Confirmation Gate ci-dessus).

## Ressources complémentaires

- [README.md](README.md) — doc utilisateur ; [MANUAL.md](MANUAL.md) — manuel des touches et format pack.
- [docs/plans/](docs/plans/) — plans par feature.
- Design et patterns repris de [rusty-console](https://github.com/TataneCode/rusty-console) (voir [AGENT_ARCHITECTURE.md](AGENT_ARCHITECTURE.md)).
