# Plan: `rapm add <dir>` / `rapm remove <name>` + kind `rules` (E13)

> Status: **approved** — branch `vibe/cli-simplifiee-config-a40da9`; gate to verify before delivery
> (`fmt`, `clippy --all-targets -- -D warnings`, `cargo test`). Source: `.evolutions/e13-ajout-par-repertoire.md`
> via skills `/challenge` then `/plan` (architect). Décision: [ADR-0001](../adr/0001-refonte-cli-config-ajout-par-repertoire.md).

## Goal

Remplacer `rapm pack create` (inputs multiples) par un ajout de pack **par répertoire**
(`rapm add <dir>` : scan de `agents/ skills/ rules/ commands/`, `pack.toml` généré) et
ajouter `rapm remove <name>` (suppression de la librairie utilisateur). `rules` devient
un kind d'item à part entière.

## Architecture decisions

- **D1: domaine — kind `rules`** — `PackItemKind::Rule` (`as_str` `"rule"`,
  `plural_dir` `"rules"`, `parse` `"rule"`) + `PackItemKind::all()` pour le scan.
  `DestinationTemplates` gagne `rule: Option<String>` et `rule_dir: Option<String>`
  (optionnels : les profils utilisateur anciens restent valides ; un item `rule` vers
  un profil sans template `rule` est simplement ignoré à l'ajout et échoue à
  l'install avec `MissingProfileTemplate`). `for_kind`/`all()` mis à jour.
- **D2: profils embarqués** — template `rule` ajouté aux 7 profils `clis/*.toml`
  (vibe `.vibe/rules/{name}.md`, gemini `.gemini/rules/{name}.md`,
  claude `.claude/rules/{name}.md`, codex `.codex/rules/{name}.md`,
  copilot `.github/instructions/{name}.instructions.md`,
  cursor `.cursor/rules/{name}.mdc`, opencode `.opencode/rules/{name}.md`) ;
  `rule_dir` non défini. Valeurs consignées comme point ouvert d'ADR-0001.
  `loader.rs` : `RawDefaults` gagne `rule`/`rule_dir` (`serde(default)`).
- **D3: application — contexte `pack_admin`** (remplace `pack_create`, supprimé) :
  - `dto.rs` : `AddPackDto { dir, name: Option<String>, overwrite }`,
    `AddPackResultDto { pack_name, pack_dir, item_count }`, `RemovePackResultDto`,
    `ScannedItem { kind: PackItemKind, host_path, pack_source, source_is_dir }`,
    `PackSourceDto`/`PackWriteOutcomeDto` (déplacés).
  - `traits.rs` : `PackWriter { write(pack, sources), remove(name) -> Result<PathBuf> }`
    et `BaseDirScanner { scan(base) -> Result<Vec<ScannedItem>> }` (tous `#[automock]`).
  - `service.rs` `PackAdminService` : `add_from_dir` — nom = `--name` ou nom du
    répertoire (`PackName`), refus si le pack existe et pas `--force`
    (`PackAlreadyExists`), scan, items canoniques `<kind>s/<nom>` avec une destination
    par profil chargé (template rendu via `CliProfile::destination`, profils sans
    template ignorés), `cli` = union des destinations, `Pack::new(..., "1.0.0", "", ...)`,
    puis `writer.write`. `remove(name)` — `PackName` validé puis `writer.remove`.
    Nouvelle erreur `ApplicationError::NoPackContent(dir)` si aucun des 4 répertoires
    n'existe.
- **D4: infrastructure** — `src/infrastructure/pack/scan.rs` : `FsBaseDirScanner` lit
  `base/<plural>/` pour chaque kind de `PackItemKind::all()`, une entrée = un item
  (fichier ou répertoire, copié récursivement), trié (kind, nom) ; `base` non
  répertoire → `NotADirectory`. `FsPackWriter` (déplacé vers `pack_admin`) gagne
  `remove(name)` : supprime `<user_root>/<name>`, `PackNotFound` si absent —
  **jamais** en dehors de la racine utilisateur. UT tempdir.
- **D5: présentation CLI** — `rapm add <dir> [--name <n>] [--force]` et
  `rapm remove <name>` dans `cli.rs` (E11), branchés sur `PackAdminService`.
- **D6: wiring** — `main.rs` : `FsPackWriter::new(library.user_root)` +
  `FsBaseDirScanner` + `FsPackRepository` → `PackAdminService` (avec le registre).

## Steps

1. **Domain** — `PackItemKind::Rule` + `all()` + UT ; `DestinationTemplates.rule/rule_dir`
  + `for_kind`/`all()` + UT ; `lib.rs` — vérif : `cargo test --lib domain`
2. **Application** — `src/application/pack_admin/*` (dto/traits/service) + suppression
   de `src/application/pack_create/` + `ApplicationError::NoPackContent` + UT mockall
   (scan → items → destinations par profil, union des clis, conflit sans `--force`,
   remove) — vérif : `cargo test --lib application`
3. **Infrastructure** — `scan.rs` (UT tempdir : 4 kinds, fichiers et répertoires,
   répertoire vide, base absente) ; `writer.rs` déplacé + `remove` (UT) ;
   `loader.rs` `rule`/`rule_dir` (UT) — vérif : `cargo test --lib infrastructure`
4. **Profils embarqués** — `clis/*.toml` : template `rule` ×7 — vérif : `cargo test --lib` (charge tous les profils embarqués)
5. **Présentation + wiring** — `cli.rs` `add`/`remove`, `main.rs` — vérif : `cargo build`
6. **IT** — `tests/pack_admin.rs` (binaire réel sur tempdir) : `rapm add ./base-pack`
   crée `<user>/base-pack/pack.toml` + contenu copié + destinations générées pour
   chaque profil ; nom par défaut = nom du répertoire, `--name` prioritaire ; pack
   existant sans `--force` → exit 1 ; `rapm remove base-pack` supprime, puis exit 1 ;
   pack avec `rules/` installable vers la destination `rule` —
   vérif : `cargo test --test pack_admin`
7. **Suppression tests obsolètes** — `tests/pack_create.rs`, `tests/cli_dry_run.rs`
   supprimés ; `tests/cli_headless.rs`, `tests/cli_profiles.rs` réécrits (E11) —
   vérif : `cargo test`
8. **Docs** — MANUAL.md (add/remove, répertoire de base, config), README.md,
   CONTEXT.md — vérif : gate complet

## Risks and mitigations

- **Deux entrées de même nom dans un kind** → `DuplicateItemSource` (règle existante).
- **Profil sans template `rule`** → item `rule` sans destination pour ce profil ;
  `cli` = union, donc le pack reste installable ailleurs.
- **`remove` sur un pack système** → impossible : n'existe pas dans la racine
  utilisateur → `PackNotFound` (la racine système est en lecture seule).

## Out of scope

- Fichiers à la racine du répertoire de base (ignorés).
- Synchronisation d'un pack ré-ajouté avec ses installations existantes.
- `pack create` (inputs multiples) : supprimé, pas de mode compatibilité.
