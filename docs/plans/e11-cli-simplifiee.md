# Plan: CLI simplifiée — commandes fondamentales uniquement (E11)

> Status: **approved** — branch `vibe/cli-simplifiee-config-a40da9`; gate to verify before delivery
> (`fmt`, `clippy --all-targets -- -D warnings`, `cargo test`). Source: `.evolutions/e11-cli-simplifiee.md`
> via skills `/challenge` then `/plan` (architect). Décision: [ADR-0001](../adr/0001-refonte-cli-config-ajout-par-repertoire.md).
> Dépend de E12 (config) et E13 (`add`/`remove`).

## Goal

Réduire la CLI headless aux commandes fondamentales, sans changer le TUI :
`rapm` (TUI), `install`, `uninstall`, `add`, `remove`, `help`.

## Architecture decisions

- **D1: surface finale** —
  ```bash
  rapm                                  # TUI (inchangé)
  rapm install <pack> [--dest <dir>] [--force]
  rapm uninstall <pack> [--dest <dir>]
  rapm add <dir> [--name <n>] [--force] # E13
  rapm remove <name>                    # E13
  rapm help | -h | --help
  ```
  Supprimés : `list`, `show`, `pack create`, `--dry-run`, `--cli`, `--dest <cli>=<path>`,
  `--vibe-dest`/`--gemini-dest`, prompts interactifs. Exit codes inchangés
  (0 succès, 1 erreur runtime, 2 usage).
- **D2: CLI cible d'installation** — plus de flag `--cli` : `install`/`uninstall`
  résolvent la CLI via `PackService::default_cli(pack)` (E12 : `default_cli` de la
  config si le pack la déclare, sinon première CLI déclarée du pack).
- **D3: parser** — `src/presentation/cli.rs` réécrit : `CliCommand::{Help, Install,
  Uninstall, Add, Remove}`, `TokenParser` conservé, `parse_*` simplifiés.
  `run(args, pack, admin)` reçoit `PackService` + `PackAdminService` (E13).
  `is_cli_invocation` inchangé (tout argument = CLI).
- **D4: service** — `PackService` garde `list`/`installed_in`/`plan_install`
  (utilisés par le TUI : panneau packs, panneau Installed, preview `P`) ;
  `detail` + `PackDetailDto`/`PackItemDto` supprimés (seul `rapm show` les utilisait).
- **D5: wiring** — `main.rs` : branche CLI construit `PackService` (avec
  `default_cli` E12) + `PackAdminService` (E13) ; `PackCreationService` supprimé.

## Steps

1. **E12 + E13 d'abord** (plans `e12-fichier-configuration.md`, `e13-ajout-par-repertoire.md`)
   — prérequis — vérif : `cargo build`
2. **CLI** — `src/presentation/cli.rs` réécrit (D1-D4) + UT parser (chaque commande,
   options, erreurs d'usage exit 2, `add`/`remove`) — vérif : `cargo test --lib presentation::cli`
3. **Service** — `PackService::detail` et DTO `PackDetailDto`/`PackItemDto` supprimés ;
   UT `default_cli` avec config — vérif : `cargo test --lib application::pack`
4. **Wiring** — `main.rs` — vérif : `cargo build`
5. **IT** — `tests/cli_headless.rs` réécrit : `install`/`uninstall` réels sur tempdir
   (exit 0, index écrit, réinstall sans `--force` → exit 1), `install` avec
   `default_cli` en config (E12), `rapm list`/`show`/`pack create`/`--dry-run`/`--cli`
   → exit 2 — vérif : `cargo test --test cli_headless`
6. **Tests supprimés** — `tests/pack_create.rs`, `tests/cli_dry_run.rs` —
   vérif : `cargo test`
7. **Docs** — MANUAL.md (usage CLI réécrit + section configuration E12), README.md,
   CONTEXT.md (statut) — vérif : `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`

## Risks and mitigations

- **Perte de `list`/`show` pour les scripts** → assumé (scope minimal, ADR-0001) ;
  le TUI reste la surface de lecture.
- **`--dry-run` supprimé mais preview TUI `P` conservé** → `plan_install` reste dans
  `PackService` pour le TUI ; seul le flag CLI disparaît.

## Out of scope

- Toute modification de l'interface TUI (conservée telle quelle, hors lecture config).
- E12 (config), E13 (add/remove) : plans séparés.
