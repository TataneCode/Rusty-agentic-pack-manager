# E03 — CLI headless : install / uninstall / list / show

- **Statut** : proposé
- **Source** : position UX — le harness engineering (scripts, CI, conteneurs de test)
  ne peut pas lancer un TUI.

## Problème

Seul `rapm pack create` existe en CLI (`src/presentation/cli.rs`). L'installation,
la désinstallation et la lecture des packs ne sont possibles qu'en TUI, donc
impossibles à scriptifier.

## Objectif

Exposer en CLI les opérations déjà servies par `PackService`, avec les mêmes
règles (validation des destinations, index `packs_index.toml`, exit codes 0/1/2) :

```bash
rapm list [--cli vibe] [--installed --dest <dir>]
rapm show <pack>
rapm install <pack> --dest <dir> [--cli <cli>] [--force]
rapm uninstall <pack> --dest <dir> [--cli <cli>]
```

## Portée

- `src/presentation/cli.rs` : nouveaux sous-commandes (parser additif, exit codes
  inchangés : 0 succès, 1 erreur runtime, 2 usage).
- `src/main.rs` : wiring du `PackService` pour le chemin CLI (aujourd'hui seul
  `PackCreationService` est construit en CLI).
- `tests/` : IT CLI via `CARGO_BIN_EXE_rapm` sur tempdir (pattern de
  `tests/pack_create.rs`).

## Hors scope

- `--dry-run` (E04), sandbox (E06), sélecteur de CLI par défaut (E07), scope user (E09).

## Critères d'acceptation

- `rapm install tdd --dest /tmp/repo --cli vibe` copie les fichiers, écrit
  `packs_index.toml`, exit 0 ; ré-installation sans `--force` → erreur, exit 1.
- `rapm list` / `show` en lecture seule, utilisables sans terminal interactif.
- Gate qualité du projet vert.
