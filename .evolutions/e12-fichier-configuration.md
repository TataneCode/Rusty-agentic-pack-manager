# E12 — Fichier de configuration (`packs_dir`, `default_cli`) lu par CLI et TUI

- **Statut** : en cours
- **Source** : consigne utilisateur + [ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md).
  Remplace le brouillon `docs/plans/pack-library-config.md` (supersédé).
- **Décision liée** : [ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md)

## Problème

L'emplacement de la librairie de packs est dérivé uniquement de l'environnement
(`$XDG_DATA_HOME/rapm/packs`, fallback `~/.local/share/rapm/packs`), et la CLI cible
des installations est la première CLI déclarée par le pack. Aucun moyen par fichier
de les fixer par utilisateur pour la CLI **et** le TUI.

## Objectif

Un fichier TOML édité à la main, lu au démarrage par la CLI et le TUI :

- Emplacement : `$XDG_CONFIG_HOME/rapm/config.toml`, fallback
  `~/.config/rapm/config.toml` (`%APPDATA%\rapm\config.toml` sous Windows).
- Schéma (toutes les clés optionnelles) :

  ```toml
  packs_dir = "/home/u/ma-librairie-de-packs"  # racine inscriptible de la librairie
  default_cli = "vibe"                          # CLI cible par défaut des installations
  ```

- `packs_dir` doit être absolu ; la librairie s'y résout comme racine utilisateur
  (première racine de lecture, racine d'écriture de `add`), les racines système
  embarquées restant en lecture seule après.
- `default_cli` est validé contre les profils chargés (`clis/*.toml`) ; s'il n'est
  pas déclaré par un pack, repli sur la première CLI déclarée du pack (point ouvert
  ADR-0001).
- Fichier absent = config vide (pas d'erreur) ; TOML malformé = erreur nommée avec
  le chemin du fichier.

## Portée

- Application : nouveau contexte `src/application/config/` (DTOs, port
  `ConfigRepository`, `ConfigService::resolve`) + `ApplicationError::InvalidConfig`.
- Infrastructure : `src/infrastructure/config/fs_config.rs` (`FsConfigRepository`,
  lecture/écriture TOML, écriture atomique) + UT sur tempdir.
- Présentation : `PackLibrary::discover_with(config)` (`src/presentation/library.rs`)
  ; wiring `src/main.rs` (résolution config avant CLI et TUI, les deux la lisent).
- CLI d'installation : `PackService`/TUI utilisent `default_cli` avant la première
  CLI déclarée du pack.
- Tests : UT priorité/résolution, IT CLI sur tempdir (`XDG_CONFIG_HOME` pointé sur un
  tempdir) ; TUI lit la config au démarrage.

## Hors scope

- Sous-commande `rapm config set/get`, `extra_roots`, variable d'env `RAPM_PACKS_DIR`,
  garde-fou TUI au démarrage, migration des packs existants (brouillon supersédé, v2).
- Écran TUI d'édition de la config.

## Critères d'acceptation

- Sans fichier de config, comportement identique à aujourd'hui (XDG par défaut).
- Avec `packs_dir` absolu, `rapm add` écrit dans ce répertoire et la CLI/TUI le
  lisent en premier.
- Avec `default_cli = "gemini"`, `rapm install <pack>` cible gemini quand le pack le
  déclare.
- `packs_dir` relatif ou TOML malformé → erreur `InvalidConfig` avec le chemin, exit 1.
- Gate qualité du projet vert.
