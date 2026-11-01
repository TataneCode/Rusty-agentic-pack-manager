# E04 — Dry-run / plan d'installation

- **Statut** : proposé
- **Source** : position UX — « tester rapidement » commence par voir ce qui serait
  fait, sans toucher au disque.

## Problème

`A` (TUI) et l'install futur (E03) copient directement et refusent si une
destination existe. Aucune prévisualisation du plan source→destination ni des
collisions.

## Objectif

Un plan d'installation calculable **sans écrire** :

- CLI : `rapm install <pack> --dest <dir> --dry-run` affiche le plan (fichier →
  destination, statut `ok` / `collision` / `hors index`) et sort 0 ; exit dédié
  (ex. 3) si collisions détectées sans `--force`.
- TUI : `P` (packs panel) ouvre une prévisualisation du plan avant `A` ; `A` reste
  l'install direct.

## Portée

- Domaine : la logique existe déjà partiellement (`plan_install` dans
  `src/domain/pack/error.rs`) — l'exposer comme résultat structuré
  (`InstallPlan { entries: Vec<PlanEntry { source, destination, conflict }> }`).
- Application : `PackService::plan_install(...) -> InstallPlanDto` (sans I/O de
  copie), `install` réutilise le même plan.
- Présentation : flag `--dry-run` (CLI) + écran/popup de plan (TUI).
- Tests : UT domaine (collisions), UT service mocké, IT CLI.

## Hors scope

- Stratégie de résolution des collisions (E05), sandbox (E06).

## Critères d'acceptation

- `--dry-run` n'écrit aucun fichier ni index (vérifié sur tempdir).
- Le plan TUI et le plan CLI affichent les mêmes entrées.
- Gate qualité vert.
