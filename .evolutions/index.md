# Evolutions — index de traçabilité

Registre des évolutions de `rapm`, déduit de la position UX (designer CLI/TUI) et de
l'audit d'architecture (frontière fronts CLI/TUI). Chaque évolution a sa fiche dédiée
dans ce répertoire.

**Règle de numérotation — le numéro E0X EST la priorité d'exécution.** Les évolutions
sont ordonnées de E01 (à faire en premier) à E14 (en dernier) ; ce fichier fait foi.
Le statut évolue : `proposé` → `planifié` (plan approuvé dans `docs/plans/`) →
`en cours` → `livré` (PR mergée) → `archivé`.

## Génération 2 — refonte CLI / config / librairie (ADR-0001)

La **génération 1 (E01–E10) est archivée** : la refonte décidée dans
l'[ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md) simplifie
drastiquement la CLI (commandes fondamentales uniquement), ajoute un fichier de
configuration lu par la CLI et le TUI, et remplace `pack create` par un ajout de pack
par répertoire. Les numéros E01–E10 restent immuables (règle ci-dessous) ; E03, E04 et
E07 sont notés supersédés.

| Priorité | Évolution | Fiche | Statut | Plan | PR | Notes |
|---|---|---|---|---|---|---|
| E01 | Verrouiller la frontière des fronts (fix dérives + test d'arch) | [e01-verrouiller-frontieres.md](e01-verrouiller-frontieres.md) | archivé (livré) | [docs/plans/e01-lock-layer-boundaries.md](../docs/plans/e01-lock-layer-boundaries.md) | [#8](https://github.com/TataneCode/Rusty-agentic-pack-manager/pull/8) | Livré en gen 1, puis archivé avec la génération |
| E02 | Profils de CLI externes (registry TOML de layouts) | [e02-profils-cli.md](e02-profils-cli.md) | archivé (livré) | [docs/plans/e02-cli-profiles.md](../docs/plans/e02-cli-profiles.md) | [#9](https://github.com/TataneCode/Rusty-agentic-pack-manager/pull/9) | Livré en gen 1 ; base des templates `rule` (E13) |
| E03 | CLI headless : install / uninstall / list / show | [e03-cli-headless.md](e03-cli-headless.md) | archivé — supersédé par E11 | [docs/plans/e03-cli-headless.md](../docs/plans/e03-cli-headless.md) | [#10](https://github.com/TataneCode/Rusty-agentic-pack-manager/pull/10) | `list`/`show` supprimés en gen 2 (ADR-0001) |
| E04 | Dry-run / plan d'installation (CLI `--dry-run` + preview TUI) | [e04-dry-run.md](e04-dry-run.md) | archivé — supersédé par E11 | [docs/plans/e04-dry-run.md](../docs/plans/e04-dry-run.md) | [#11](https://github.com/TataneCode/Rusty-agentic-pack-manager/pull/11) | `--dry-run` CLI supprimé en gen 2 (ADR-0001) |
| E05 | Stratégie de conflits à l'install (refuse/skip/overwrite/suffix) | [e05-conflits.md](e05-conflits.md) | archivé | — | — | Non repris en gen 2 ; défaut `refuse` = comportement actuel |
| E06 | `rapm pack try <pack>` — sandbox jetable de test | [e06-pack-try.md](e06-pack-try.md) | archivé | — | — | Non repris en gen 2 |
| E07 | Sélecteur de CLI à l'install (popup TUI + `--cli`) | [e07-selecteur-cli.md](e07-selecteur-cli.md) | archivé — supersédé par E12 | — | — | Remplacé par le `default_cli` de la config (ADR-0001) |
| E08 | Dépendances entre packs (`depends` + check à l'install) | [e08-depends.md](e08-depends.md) | archivé | — | — | Non repris en gen 2 |
| E09 | Scope `user` d'installation (`--scope user\|project`, index global) | [e09-scope-user.md](e09-scope-user.md) | archivé | — | — | Non repris en gen 2 |
| E10 | Détection de drift + `upgrade` (index vs disque vs version) | [e10-drift-upgrade.md](e10-drift-upgrade.md) | archivé | — | — | Non repris en gen 2 |
| E11 | CLI simplifiée : commandes fondamentales uniquement | [e11-cli-simplifiee.md](e11-cli-simplifiee.md) | en cours | [docs/plans/e11-cli-simplifiee.md](../docs/plans/e11-cli-simplifiee.md) | — | Supprime `list`/`show`/`pack create`/`--dry-run`/`--cli` ; ADR-0001 |
| E12 | Fichier de configuration (`packs_dir`, `default_cli`) lu par CLI et TUI | [e12-fichier-configuration.md](e12-fichier-configuration.md) | en cours | [docs/plans/e12-fichier-configuration.md](../docs/plans/e12-fichier-configuration.md) | — | Remplace le brouillon `pack-library-config.md` ; ADR-0001 |
| E13 | `rapm add <dir>` / `rapm remove <name>` + kind `rules` | [e13-ajout-par-repertoire.md](e13-ajout-par-repertoire.md) | en cours | [docs/plans/e13-ajout-par-repertoire.md](../docs/plans/e13-ajout-par-repertoire.md) | — | Scan `agents/ skills/ rules/ commands/` → `pack.toml` généré ; ADR-0001 |
| E14 | Log ADR (`docs/adr/`) + archivage de la génération 1 | [e14-log-adr.md](e14-log-adr.md) | en cours | [ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md) | — | Ce changement même |

## Génération 1 — correspondance avec l'ancienne numérotation (archivée)

La première rédaction ordonnait les fiches par thème ; la priorité a ensuite été
revue (décision UX : les profils de CLI remontent en E02). Ancien → nouveau :

| Ancien | Nouveau (gen 1) | Évolution |
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
- Une évolution à impact d'architecture est documentée par une ADR dans
  [docs/adr/](../docs/adr/README.md) et y fait référence.
