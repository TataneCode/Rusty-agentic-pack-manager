# Plan: fichier de configuration (`packs_dir`, `default_cli`) lu par la CLI et le TUI (E12)

> Status: **approved** — branch `vibe/cli-simplifiee-config-a40da9`; gate to verify before delivery
> (`fmt`, `clippy --all-targets -- -D warnings`, `cargo test`). Source: `.evolutions/e12-fichier-configuration.md`
> via skills `/challenge` then `/plan` (architect). Décision d'architecture: [ADR-0001](../adr/0001-refonte-cli-config-ajout-par-repertoire.md).
> Remplace le brouillon `pack-library-config.md` (supersédé).

## Goal

Un fichier TOML édité à la main, lu au démarrage par la CLI et le TUI, avec deux clés :
`packs_dir` (racine inscriptible de la librairie de packs) et `default_cli` (CLI cible
par défaut des installations).

## Architecture decisions

- **D1: emplacement** — `$XDG_CONFIG_HOME/rapm/config.toml`, fallback
  `~/.config/rapm/config.toml` (`%APPDATA%\rapm\config.toml` sous Windows). Fichier
  absent = config vide (pas d'erreur) ; TOML malformé = `ApplicationError::InvalidConfig`
  avec le chemin. Pas de sous-commande `rapm config`, pas de `extra_roots`, pas de
  variable d'env (v2).
- **D2: schéma** — toutes les clés optionnelles :
  `packs_dir = "/abs/path"` (doit être absolu), `default_cli = "vibe"` (validé contre
  le registre des profils au wiring).
- **D3: domaine** — nouveau VO `LibraryPath` dans `src/domain/config/value_objects.rs`
  (chemin absolu non vide, `~` refusé) + `DomainError::InvalidLibraryPath(String)`.
  Pur, testé unitairement.
- **D4: application** — nouveau contexte `src/application/config/` : `AppConfigDto
  { packs_dir: Option<PathBuf>, default_cli: Option<String> }`, port `#[automock]
  ConfigRepository { fn load() -> Result<AppConfigDto, ApplicationError> }`,
  `ConfigService::load()` (validation `packs_dir` via `LibraryPath`, `default_cli`
  via `CliId::new`) + `ApplicationError::InvalidConfig { path, message }`.
- **D5: infrastructure** — `src/infrastructure/config/fs_config.rs` :
  `FsConfigRepository::new(path)` + `FsConfigRepository::discover()` (chemin par défaut
  depuis l'env, `Option`). Lecture seule : `std::fs::read_to_string` + `toml`, aucune
  écriture. UT sur tempdir (absent, malformé, valide, `packs_dir` relatif rejeté).
- **D6: présentation** — `PackLibrary::discover_with(&AppConfigDto)` dans
  `src/presentation/library.rs` : `user_root = config.packs_dir` sinon XDG par défaut ;
  les racines de profils CLI restent dérivées de la racine XDG (stables), pas de
  `packs_dir`. `discover()` = `discover_with(&default)` (compat tests).
  `PackService::with_default_cli(Option<String>)` + `default_cli(pack)` : config si le
  pack la déclare, sinon première CLI déclarée, sinon `"vibe"` (inchangé sinon).
- **D7: wiring** — `main.rs` : `FsConfigRepository::discover()` → `ConfigService::load`
  → validation de `default_cli` contre le registre (anyhow au bord) →
  `PackLibrary::discover_with` → `build_pack_service(library, default_cli)` pour la CLI
  **et** le TUI. Le TUI ne change pas de code : il passe par `pack_service.default_cli`.

## Steps

1. **Domain** — `src/domain/config/{mod,value_objects}.rs` : `LibraryPath` +
   `DomainError::InvalidLibraryPath` + UT ; `lib.rs` enregistre le module —
   vérif : `cargo test --lib domain`
2. **Application** — `src/application/config/{mod,dto,traits,service}.rs` +
   `ApplicationError::InvalidConfig` + UT mockall — vérif : `cargo test --lib application::config`
3. **Infrastructure** — `fs_config.rs` + UT tempdir — vérif : `cargo test --lib infrastructure::config`
4. **Presentation** — `library.rs::discover_with` + UT matrice (config vide/pleine,
   XDG) ; `PackService::with_default_cli` + UT (config supportée / non supportée /
   absente) — vérif : `cargo test --lib presentation`, `cargo test --lib application::pack`
5. **Wiring** — `main.rs` (config → library → services, CLI et TUI) —
   vérif : `cargo build`
6. **IT** — `tests/config_cli.rs` (binaire réel, `XDG_CONFIG_HOME` sur tempdir) :
   sans config = comportement XDG ; `packs_dir` absolu → `rapm add` écrit là et la
   liste TUI/CLI le lit ; `default_cli = "gemini"` → `rapm install` cible gemini ;
   `packs_dir` relatif → exit 1 avec le chemin — vérif : `cargo test --test config_cli`
7. **Docs** — MANUAL.md (section configuration), README.md, CONTEXT.md (statut) —
   vérif : liens valides

## Risks and mitigations

- **Config invalide bloque le TUI** → `InvalidConfig` nommée avec le chemin, exit 1
  en CLI ; en TUI, popup d'erreur au démarrage (jamais de panic, `rules/presentation.md`).
- **Anciens packs invisibles après changement de `packs_dir`** → documenté dans
  MANUAL.md (migration manuelle, point ouvert ADR-0001).
- **Windows** → branche `%APPDATA%` miroir de `user_packs_dir_from`.

## Out of scope

- `rapm config set/get`, `extra_roots`, `RAPM_PACKS_DIR`, garde-fou TUI, migration
  automatique (v2, brouillon supersédé).
- Écran TUI d'édition de la config.
