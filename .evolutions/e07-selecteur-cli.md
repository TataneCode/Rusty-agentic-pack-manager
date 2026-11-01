# E07 — Sélecteur de CLI à l'installation

- **Statut** : proposé
- **Source** : `MANUAL.md` note déjà « a CLI picker is planned for v1.1 » ;
  `src/presentation/tui/router.rs:235` installe toujours la première CLI déclarée.

## Problème

Un pack `cli = ["vibe", "gemini"]` s'installe systématiquement pour la première CLI.
L'utilisateur ne choisit jamais, en TUI comme (bientôt) en CLI.

## Objectif

- TUI : à l'install (`A`), si le pack déclare plusieurs CLIs, popup de sélection
  (liste des CLIs supportées, `Enter` valide). Si une seule CLI : install direct,
  comportement actuel.
- CLI : `rapm install <pack> --cli gemini` (E03) ; sans `--cli`, même règle que le
  TUI (première déclarée) mais **documentée**, ou erreur demandant `--cli` si
  ambigu — à trancher au plan.
- La décision « CLI par défaut » migre dans `PackService` (préreq. E01) : le router
  ne choisit plus.

## Portée

- `src/application/pack/` : `default_cli` exposé en DTO (`String`), règle testée.
- `router.rs` + `common/widgets.rs` : popup de sélection (réutilise `SelectList`).
- `cli.rs` : flag `--cli` sur `install`/`uninstall`.
- MANUAL.md : mise à jour de la section Installation.

## Hors scope

- Détection auto du CLI présent dans le projet (existence de `.gemini/`) — possible
  v2 pour pré-sélectionner le popup, jamais pour décider seul.

## Critères d'acceptation

- Pack multi-CLIs : popup TUI affiché, install pour la CLI choisie ; index enregistre
  le bon `cli`.
- Pack mono-CLI : aucun popup, comportement inchangé.
- TestBackend TUI + IT CLI.
- Gate qualité vert.
