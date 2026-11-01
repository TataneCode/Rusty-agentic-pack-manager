# Evolutions — index de traçabilité

Registre des évolutions de `rapm`, déduit de la position UX (designer CLI/TUI) et de
l'audit d'architecture (frontière fronts CLI/TUI). Chaque évolution a sa fiche dédiée
dans ce répertoire.

**Règle de numérotation — le numéro E0X EST la priorité d'exécution.** Les évolutions
sont ordonnées de E01 (à faire en premier) à E10 (en dernier) ; ce fichier fait foi.
Le statut évolue : `proposé` → `planifié` (plan approuvé dans `docs/plans/`) →
`en cours` → `livré` (PR mergée) → `archivé`.

| Priorité | Évolution | Fiche | Statut | Plan | PR | Notes |
|---|---|---|---|---|---|---|
| E01 | Verrouiller la frontière des fronts (fix dérives + test d'arch) | [e01-verrouiller-frontieres.md](e01-verrouiller-frontieres.md) | en cours | [docs/plans/e01-lock-layer-boundaries.md](../docs/plans/e01-lock-layer-boundaries.md) | [#8](https://github.com/TataneCode/Rusty-agentic-pack-manager/pull/8) | Prérequis de toutes les autres. Audit : 2 imports `01_domain` en présentation (`app.rs:7`, `router.rs:1`), décision CLI dans `router.rs:235`, I/O direct dans `library.rs:40` |
| E02 | Profils de CLI externes (registry TOML de layouts) | [e02-profils-cli.md](e02-profils-cli.md) | en cours | [docs/plans/e02-cli-profiles.md](../docs/plans/e02-cli-profiles.md) | [#9](https://github.com/TataneCode/Rusty-agentic-pack-manager/pull/9) | Placé tôt (décision UX) : `TargetCli` enum → newtype `CliId` + 7 profils shippés (vibe, gemini, claude, copilot, opencode, codex, cursor) surchargeables ; impacte config, packaging, installs. Débloque E03/E07 data-driven |
| E03 | CLI headless : install / uninstall / list / show | [e03-cli-headless.md](e03-cli-headless.md) | en cours | [docs/plans/e03-cli-headless.md](../docs/plans/e03-cli-headless.md) | [#10](https://github.com/TataneCode/Rusty-agentic-pack-manager/pull/10) | Base du harness engineering (scripts, CI) |
| E04 | Dry-run / plan d'installation (CLI `--dry-run` + prévisu TUI) | [e04-dry-run.md](e04-dry-run.md) | proposé | — | — | « Tester avant d'installer » sans toucher au disque |
| E05 | Stratégie de conflits à l'install (refuse/skip/overwrite/suffix) | [e05-conflits.md](e05-conflits.md) | proposé | — | — | Défaut `refuse` = comportement actuel |
| E06 | `rapm pack try <pack>` — sandbox jetable de test | [e06-pack-try.md](e06-pack-try.md) | proposé | — | — | Harness rapide : install isolé, rapport, nettoyage |
| E07 | Sélecteur de CLI à l'install (popup TUI + `--cli`) | [e07-selecteur-cli.md](e07-selecteur-cli.md) | proposé | — | — | Remplace « première CLI déclarée » (`router.rs:235`), noté v1.1 dans MANUAL.md |
| E08 | Dépendances entre packs (`depends` + check à l'install) | [e08-depends.md](e08-depends.md) | proposé | — | — | Aujourd'hui en texte libre (`vibe-wise-commands`) |
| E09 | Scope `user` d'installation (`--scope user\|project`, index global) | [e09-scope-user.md](e09-scope-user.md) | proposé | — | — | « user » ne s'applique aujourd'hui qu'à la bibliothèque de packs |
| E10 | Détection de drift + `upgrade` (index vs disque vs version) | [e10-drift-upgrade.md](e10-drift-upgrade.md) | proposé | — | — | `packs_index.toml` existe, pas de comparaison post-install |

## Correspondance avec l'ancienne numérotation (traçabilité)

La première rédaction ordonnait les fiches par thème ; la priorité a ensuite été
revue (décision UX : les profils de CLI remontent en E02). Ancien → nouveau :

| Ancien | Nouveau | Évolution |
|---|---|---|
| E01 | E01 | Verrouiller la frontière des fronts |
| E08 | E02 | Profils de CLI externes |
| E02 | E03 | CLI headless |
| E03 | E04 | Dry-run / plan |
| E10 | E05 | Stratégie de conflits |
| E04 | E06 | `pack try` |
| E05 | E07 | Sélecteur de CLI |
| E06 | E08 | Dépendances entre packs |
| E07 | E09 | Scope user |
| E09 | E10 | Drift + upgrade |

## Règles de ce registre

- Une évolution ne passe à `planifié` qu'avec un plan approuvé dans `docs/plans/`
  (règle `/plan` du projet).
- Une évolution ne passe à `livré` qu'après merge de la PR ; le lien PR est renseigné.
- Le numéro de la fiche est immuable une fois attribué : une évolution annulée garde
  son numéro (statut `archivé`), on ne renumérote jamais.
