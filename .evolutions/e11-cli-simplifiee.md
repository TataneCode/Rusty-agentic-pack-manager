# E11 — CLI simplifiée : commandes fondamentales uniquement

- **Statut** : planifié
- **Source** : consigne utilisateur — la CLI a dérivé, `pack create` est trop complexe.
- **Décision liée** : [ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md)

## Problème

La CLI expose `list` / `show` / `install` / `uninstall` / `pack create` avec une
matrice d'inputs multiples (`--skill`/`--agent`/`--command` répétés, `--dest
<cli>=<path>` par item, prompts interactifs). L'usage fondamental — installer ou
désinstaller un pack existant dans la librairie — est noyé dans cette surface.

## Objectif

```bash
rapm                                  # TUI (inchangé)
rapm install <pack> [--dest <dir>] [--force]
rapm uninstall <pack> [--dest <dir>]
rapm add <dir> [--name <n>] [--force] # E13
rapm remove <name>                    # E13
rapm help
```

Suppression de `list`, `show`, `pack create`, `--dry-run`, `--cli` et des prompts
interactifs. La CLI cible d'installation vient du `default_cli` de la config (E12) ;
à défaut, de la première CLI déclarée par le pack (comportement antérieur).

## Portée

- `src/presentation/cli.rs` : parser réécrit autour des 4 commandes + `help` ;
  `USAGE` réécrit ; exit codes inchangés (0 succès, 1 erreur runtime, 2 usage).
- `src/application/pack_create/` supprimé (remplacé par E13) ; `cli::run` ne reçoit
  plus de `PackCreationService`.
- `src/application/pack/service.rs` : résolution de la CLI cible via la config (E12).
- Tests : parser UT réécrits ; `tests/cli_headless.rs`, `tests/cli_dry_run.rs`,
  `tests/pack_create.rs` supprimés ou réécrits ; IT `install`/`uninstall` conservés.
- `MANUAL.md`, `README.md` : usage CLI mis à jour.

## Hors scope

- E12 (fichier de configuration), E13 (`add`/`remove` + kind `rules`).
- Toute modification de l'interface du TUI (conservé tel quel, hors lecture config).

## Critères d'acceptation

- `rapm install tdd --dest /tmp/repo` installe (CLI = `default_cli` config ou première
  CLI du pack), exit 0 ; réinstallation sans `--force` → erreur, exit 1.
- `rapm uninstall tdd --dest /tmp/repo` supprime les chemins indexés, exit 0.
- `rapm list`, `rapm show tdd`, `rapm pack create …`, `--dry-run`, `--cli` → erreur
  d'usage, exit 2.
- Gate qualité du projet vert (`fmt`, `clippy -D warnings`, `test`).
