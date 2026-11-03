# E14 — Log ADR (`docs/adr/`) + archivage de la génération 1

- **Statut** : en cours
- **Source** : consigne utilisateur — prévoir un Architecture Decision Log suite à la
  refonte majeure, et recréer la liste d'évolutions.
- **Décision liée** : [ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md)

## Problème

Les décisions d'architecture majeures (ici : refonte de la CLI, fichier de
configuration, ajout de pack par répertoire) n'avaient pas de registre dédié ; le
registre `.evolutions` (génération 1, E01–E10) était devenu obsolète.

## Objectif

- `docs/adr/` : conventions du log (numéros immuables, statuts, format) dans
  `README.md` + première fiche [ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md)
  actant la refonte.
- `.evolutions/index.md` réécrit en **génération 2** : E01–E10 archivés (E03/E04/E07
  notés supersédés), nouvelles évolutions E11–E14.
- Pointeurs à jour : `AGENTS.md` et `CONTEXT.md` référencent `docs/adr/`.

## Portée

- `docs/adr/README.md`, `docs/adr/0001-*.md` (nouveaux).
- `.evolutions/index.md` (réécrit), `.evolutions/e11…e14` (nouvelles fiches) ; les
  fiches E01–E10 restent en historique (numéros immuables).
- `docs/plans/pack-library-config.md` : statut supersédé (remplacé par E12).
- `AGENTS.md`, `CONTEXT.md` : lignes « sources of truth »/navigation mises à jour.

## Hors scope

- Le code des évolutions E11–E13 (leur propre changement).

## Critères d'acceptation

- `docs/adr/README.md` liste ADR-0001 ; ADR-0001 couvre contexte/décision/conséquences
  de la refonte.
- `.evolutions/index.md` : génération 2 lisible, E01–E10 archivés, E11–E14 présentes.
- Tous les liens Markdown du dépôt pointent vers des fichiers existants.
