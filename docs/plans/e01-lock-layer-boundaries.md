# Plan: E01 — Verrouiller la frontière des fronts (CLI + TUI)

> Status: **draft** — awaiting user approval. Source: `.evolutions/e01-verrouiller-frontieres.md` (audit d'architecture, point 1) via skills `/challenge` then `/plan` (architect). Format: `rust-architecture-planning`. User decisions are marked ✅.

## Challenge (framing, per `/challenge`)

### Restatement

L'audit d'architecture constate que la règle « strictly inward » (`rules/README.md`) est écrite mais non verrouillée : 2 imports `01_domain` dans `04_presentation` (`app.rs:7`, `router.rs:1`), la décision « CLI par défaut » vit dans le router TUI (`router.rs:235`), et `src/presentation/library.rs:40` fait du `std::fs` direct. E01 rend ces dérives impossibles silencieusement : DTOs sans types du domaine, décision de CLI par défaut déplacée en application, I/O derrière un port, et un test d'architecture qui fait échouer le build en cas de régression.

### Blocking ambiguities (settled by decision, marked ✅)

1. **DTO `Vec<String>` ou newtype `CliId(String)` ?** ✅ `String` (valeurs `TargetCli::as_str()`, ex. `"vibe"`) : contrat neutre minimal, pas de nouveau type du domaine exposé ; un newtype DTO serait un deuxième type à maintenir pour aucun gain à ce stade (E02 introduira les profils de CLI proprement).
2. **`library.rs` déplacé hors de `04_presentation` ou I/O derrière un port ?** ✅ Port `LibraryInitializer` (trait application + adapter infrastructure, wiring `main.rs`) : plus petit diff que le déplacement du fichier ; `PackLibrary` reste de la résolution de configuration, pas un view. ✅ Tranché au plan comme prévu par la fiche.
3. **Tests co-localisés de la présentation : la règle d'imports s'y applique-t-elle ?** ✅ Oui, sans exception : les helpers de test d'`app.rs` construiront les DTOs en `String`. `rules/presentation.md` est amendé en ce sens. Seule la règle « pas de `std::fs` » tolère `#[cfg(test)]` pour `library.rs` (fixtures `TempDir`).

### Detected scope creep (excluded)

- E02–E10 (toute évolution fonctionnelle) — hors scope de la fiche.
- Refonte du wiring `main.rs` au-delà du remplacement de l'appel `ensure_user_root`.
- Changement de comportement install/uninstall (critère d'acceptation : comportement identique).

### Proposed scope

- **Minimal = full** ici : les 4 correctifs de la fiche (imports, décision CLI, I/O, test d'arch) forment un seul refactor indivisible pour garder le repo compilable à chaque commit.

### Decision requested

✅ Go on the scope above.

---

## Goal

Rendre la frontière « strictly inward » infranchissable silencieusement : `src/04_presentation/` n'importe plus `01_domain`/`03_infrastructure` (y compris dans ses tests co-localisés) et ne fait plus d'I/O direct, la décision de CLI par défaut migre dans `PackService`, et un test d'architecture `tests/architecture.rs` fait échouer le build en cas de régression.

## Architecture decisions

- **D1: DTOs en `String`, pas en `TargetCli`** — `PackSummaryDto.clis: Vec<TargetCli>` → `Vec<String>`, `InstalledPackDto.cli` et `UninstallResultDto.cli` → `String` (valeurs `TargetCli::as_str()`). Rationale : la règle « UI only sees DTOs » interdit tout type du domaine dans un DTO ; le mappage reste côté application (`PackSummaryDto::from_pack`, `InstalledPackDto::from_installed`).
- **D2: Signatures service en `&str` + `default_cli`** — `PackService::install`/`uninstall` prennent `cli: &str`, parsé en interne via `TargetCli::parse` (erreur → `ApplicationError::Domain(UnknownCli)`). Nouveau `PackService::default_cli(pack_name) -> Result<String, ApplicationError>` : « première CLI déclarée », fallback `"vibe"` (comportement actuel préservé), testable avec `mockall`. Rationale : la règle métier « quelle CLI par défaut » quitte le router (`router.rs:235`) pour la couche application.
- **D3: Affichage des CLI en présentation** — `split_view.rs` affiche les ids DTO (`pack.clis.first()`, `entry.cli`) avec une capitalisation de pure présentation (helper local, ex. « Vibe » au lieu de « Vibe CLI »). `ConfirmKind::Uninstall.cli` et `Mode::Installing.cli` (`app.rs`) passent en `String`.
- **D4: I/O de `library.rs` derrière un port** — nouveau trait `#[automock] LibraryInitializer` dans `src/application/library/traits.rs` (`fn ensure_user_root(&self, root: &Path) -> Result<(), ApplicationError>`), implémenté par `FsLibraryInitializer` dans `src/infrastructure/filesystem/library.rs` (`create_dir_all`, erreur → `ApplicationError::Io`). `PackLibrary::ensure_user_root` est supprimé ; `main.rs` (point de composition DI) appelle l'initializer. `PackLibrary` reste en presentation mais devient pur hors tests.
- **D5: Test d'architecture `tests/architecture.rs`** (IT, scan du vrai `src/`) :
  1. aucun fichier de `src/04_presentation/**` (tests co-localisés compris) ne contient `use crate::domain` / `use crate::infrastructure` ;
  2. aucun fichier de `src/04_presentation/tui/**` ne fait d'I/O direct (`std::fs::`) ; `src/presentation/library.rs` idem hors `#[cfg(test)]` ;
  3. aucun fichier de `src/02_application/**` n'importe `crate::infrastructure`.
  UT de la fonction de parsing sur des extraits en mémoire.
- **D6: `rules/presentation.md` amendé** — la règle « no import of `01_domain`/`03_infrastructure` » s'applique explicitement aussi aux `use` des modules `#[cfg(test)]` co-localisés de la présentation. `rules/quality.md` : le gate liste explicitement `cargo test --test architecture`.
- **D7: Tests** — UT `mockall` du service mis à jour (`"vibe"` à la place de `TargetCli::Vibe`) + UT `default_cli` (première CLI déclarée, fallback `"vibe"`) ; IT `tests/pack_install.rs` / `tests/split_screen_ui.rs` adaptés aux signatures `&str` et DTO `String` (les `tests/` sont hors couches : le domaine y reste autorisé). Aucune assertion modifiée « pour passer » : elles suivent le contrat.

## Steps

1. **Service + DTOs + présentation + tests** (un seul commit atomique : le refactor DTO touche les trois couches) — fichiers : `src/application/pack/dto.rs`, `src/application/pack/service.rs` (+ UT), `src/presentation/tui/app.rs` (imports, `ConfirmKind`, `Mode`, `filter_available`, tests co-localisés), `src/presentation/tui/router.rs` (`install_here` via `default_cli`, `execute_uninstall`), `src/presentation/tui/split_view.rs` (labels), `tests/pack_install.rs`, `tests/split_screen_ui.rs` — vérification : `cargo test`.
2. **Port `LibraryInitializer`** — fichiers : `src/application/library/traits.rs` (nouveau), `src/lib.rs`, `src/infrastructure/filesystem/library.rs` (nouveau, UT sur `TempDir`), `src/presentation/library.rs` (suppression de `ensure_user_root` et du `std::fs` hors tests), `src/main.rs` (wiring) — vérification : `cargo test`.
3. **Test d'architecture** — fichier : `tests/architecture.rs` (nouveau) — vérification : `cargo test --test architecture` (rouge si lancé avant le step 1, vert après).
4. **Docs & registre** — fichiers : `rules/presentation.md`, `rules/quality.md`, `.evolutions/index.md` (E01 → `livré` à la livraison, lien PR renseigné), ce plan (statut → implemented) — vérification : `grep -rn "crate::domain\|crate::infrastructure" src/presentation/` → 0 résultat.
5. **Gate final** — `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`.

## Risks and mitigations

- **Changement cosmétique des labels CLI** (« Vibe CLI » → « Vibe ») → mitigation D3 ; les assertions IT concernées suivent le nouveau contrat (documenté ici).
- **Step 1 gros** (3 couches) → indivisible pour garder le repo compilable à chaque commit ; une seule vérification `cargo test` à la fin du step.
- **Scan d'architecture trop strict** (faux positifs sur `#[cfg(test)]`) → seule la règle 2 tolère les blocs test, et uniquement pour `library.rs` ; la règle d'imports (1) couvre tout, tests compris (D6).

## Out of scope

- E02–E10 (toute évolution fonctionnelle).
- Refonte du wiring `main.rs` au-delà du remplacement de l'appel `ensure_user_root`.
- Tout changement de comportement visible (install/uninstall identiques).

## Critères d'acceptation (rappel de la fiche)

- `grep -rn "crate::domain\|crate::infrastructure" src/presentation/` → 0 résultat.
- `cargo test --test architecture` vert et intégré au gate `rules/quality.md`.
- Aucun changement de comportement visible (install/uninstall identiques).
- Gate : `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
