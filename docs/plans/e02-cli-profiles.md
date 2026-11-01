# Plan: E02 — Profils de CLI externes (registry TOML de layouts)

> Status: **approved, in progress** — branch `vibe/e02-cli-profiles-460e33`; gate to verify before delivery (`fmt`, `clippy --all-targets -- -D warnings`, `cargo test`). Source: `.evolutions/e02-profils-cli.md` via skills `/challenge` then `/plan` (architect). Format: `rust-architecture-planning`. User decisions are marked ✅.

## Challenge (framing, per `/challenge`)

### Restatement

La connaissance « où va un skill/agent/command pour la CLI X » est codée dans chaque `pack.toml` (destinations explicites par CLI) et `TargetCli` est un enum fermé à 2 variantes (`src/domain/pack/value_objects.rs:8`) : ajouter une CLI = recompile + mise à jour des 16 manifests. E02 externalise ces layouts dans des profils TOML chargeables (shippés avec rapm, surchargeables via la user library, même ordre de priorité que les packs) et remplace l'enum par un newtype `CliId(String)` validé contre le registry des profils.

### Blocking ambiguities (settled by decision, marked ✅)

1. **Quels profils shipper ?** ✅ 7 profils : `vibe`, `gemini` (layouts observés dans les 16 packs) + `claude`, `copilot`, `opencode`, `codex`, `cursor` (conventions usuelles, surchargeables).
2. **Migration enum → newtype en un coup ou progressive ?** ✅ En une seule fois (big bang sur une branche), la « migration douce » de la fiche ne concerne que les manifests.

### Detected scope creep (excluded)

- Auto-détection des CLIs installées (`which claude`) — v2, hors scope fiche.
- Fallback universel Agent Skills — hors scope fiche.
- Modification des 16 `packs/*/pack.toml` — leurs destinations explicites restent inchangées et priment sur les profils.

### Proposed scope

- **Minimal = full** : bounded context `cli_profile` (domaine/application/infrastructure), migration `TargetCli` → `CliId`, 7 profils shippés, résolution explicite > profil à l'install, CLI `pack create` généralisée, IT des critères d'acceptation, gate vert.

### Decision requested

✅ Go on the scope above.

---

## Goal

Rendre l'ajout d'une CLI externe possible sans recompile : `TargetCli` (enum fermé) devient `CliId(String)` validé contre un registry de **profils TOML** (`clis/<name>.toml`, racines = user library → exe → cwd → système, priorité identique aux packs), et un pack peut déclarer `cli = ["x"]` avec des items sans destinations explicites, calculées depuis le profil (`{name}` = nom de l'item). Rétrocompatible : destinations explicites inchangées, 16 packs existants installent sans modification.

## Architecture decisions

- **D1: newtype `CliId(String)`** dans un nouveau bounded context `01_domain/cli_profile` (validation type `PackName` : lowercase alnum + `-`/`_`, ≤64), remplaçant `TargetCli` partout en un seul coup (domain, application, infrastructure, tests). L'ancienne liste « CLIs connues » = le registry des profils chargés.
- **D2: profils sur disque, priorité identique aux packs** — racines = `$XDG_DATA_HOME/rapm/clis` → `clis/` next to binary → `clis/` in cwd → `/usr/share/rusty-agentic-pack-manager/clis` ; premier profil de même nom gagne (user override ship). Pas d'`include_str!` : un ship sans recompile est l'objectif.
- **D3: résolution « explicite > profil » en application** — port `CliProfileRepository` + service `CliProfileService::resolve_pack(&Pack, &CliId) -> Result<Pack, ApplicationError>` ; `PackService::install` résout le pack avant `plan_install` (domaine inchangé côté logique). Les adapters infra ne décident pas.
- **D4: templates à placeholders `{name}` / `{file}`**, par kind et nature de source : `skill`/`agent`/`command` (source fichier) + `skill_dir`/`agent_dir`/`command_dir` optionnels (source répertoire). Nom d'item : répertoire source → son nom ; fichier → nom du parent (`SKILL.md` → dossier parent) sinon stem du fichier. Le domaine calcule la destination via `CliProfile::destination(...)` (fonction pure) en réutilisant `validate_item_destination`. `source_is_dir` obtenu via une nouvelle méthode du port `PackContentRepository` (implémentée par `FsPackContentRepository`) — pas d'I/O dans domaine/application.
- **D5: validation au chargement** — `FsPackRepository::new(roots, profiles)` : toute `cli` ou clé de destination sans profil → `DomainError::UnknownCliProfile`. Les destinations explicites restent prioritaires.
- **D6: 7 profils shippés** dans `clis/` ; packaging CI (RPM/DEB/tar.gz/zip) embarque `clis/` comme `packs/`.
- **D7: CLI `pack create` généralisée** — `--dest <cli>=<path>` générique (`--vibe-dest`/`--gemini-dest` conservés pour rétrocompat), `--cli` accepte tout id de profil.

## Steps

1. **Domaine `cli_profile` + migration `CliId`** — nouveaux fichiers `src/01_domain/cli_profile/{mod.rs,value_objects.rs,profile.rs}` (`CliId`, `CliProfile`, `CliProfileRegistry`, calcul de destination) ; nouveaux variants `DomainError::{InvalidCliId, UnknownCliProfile, MissingProfileTemplate, InvalidProfileTemplate}` ; `TargetCli` → `CliId` dans `src/01_domain/pack/{value_objects,entity,error,installed}.rs` + UT co-localisés ; `src/lib.rs`. Vérif : `cargo test --lib`.
2. **Application** — nouveaux `src/02_application/cli_profile/{mod.rs,traits.rs,service.rs}` (port `#[automock] CliProfileRepository`, `CliProfileService` : explicite > profil, erreur si aucun) ; migration `CliId` dans `application/pack/{traits,service,dto}.rs` et `application/pack_create/service.rs` ; `PackContentRepository::source_is_dir` ; `PackService::install` appelle `resolve_pack` avant `plan_install` ; UT mockall co-localisés. Vérif : `cargo test --lib`.
3. **Infrastructure** — nouveau `src/03_infrastructure/cli_profile/loader.rs` (`FsCliProfileRepository` : racines ordonnées, priorité user > ship, profils malformés skippés, UT tempdir) ; migration `CliId` dans `infrastructure/pack/{loader,index,writer}.rs` + validation registry au chargement ; `FsPackContentRepository::source_is_dir`. Vérif : `cargo test --lib`.
4. **Presentation + wiring** — `src/presentation/library.rs` : `profile_roots` (mêmes règles que `roots`, dossier `clis`, UT purs) ; `src/presentation/cli.rs` : `--dest <cli>=<path>` + USAGE ; `src/main.rs` : DI du registry, passage à `FsPackRepository` et `PackService`. Vérif : `cargo test --lib presentation`.
5. **Profils shippés + packaging + doc** — `clis/{vibe,gemini,claude,copilot,opencode,codex,cursor}.toml` ; `Cargo.toml` (metadata RPM/DEB), `.github/workflows/{ci,release}.yml`, `packaging/expand_rpm_assets.py` (ajout `clis/`) ; `MANUAL.md` (section profils + override user). Vérif : `cargo check`.
6. **IT** — `tests/cli_profiles.rs` : (a) pack `cli = ["claude"]` sans destinations + profil claude → install OK ; (b) profil user override profil ship ; (c) boucle sur les 16 vrais `packs/*/pack.toml` + vrais `clis/` → `plan_install` OK pour chaque CLI déclarée. `tests/pack_install.rs` migré `CliId`. Vérif : `cargo test --test cli_profiles && cargo test --test pack_install`.
7. **Gate + registre** — `cargo fmt` → `cargo clippy --all-targets -- -D warnings` → `cargo test` → `cargo check` ; `.evolutions/index.md` : E02 → `en cours` puis `livré` à la livraison (lien PR).

## Risks and mitigations

- **Régression des 16 packs** → destinations explicites prioritaires + IT (c) sur les vrais manifests.
- **Big bang enum→newtype** → étapes ordonnées domaine→application→infra→présentation, `cargo test --lib` après chaque étape.
- **Layouts des 5 nouvelles CLIs = conventions externes** → surchargeables via user library, documenté dans MANUAL.md.
- **MSRV 1.88 / clippy `-D warnings`** → gate étape 7 avant PR.

## Out of scope

Auto-détection `which <cli>`, fallback universel Agent Skills, sélecteur de CLI à l'install (E07), dry-run (E04), modification des `packs/*/pack.toml`, label de CLI en TUI (les DTOs portent déjà des `String`).
