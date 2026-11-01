# E01 — Verrouiller la frontière des fronts (CLI + TUI)

- **Statut** : proposé
- **Source** : audit d'architecture (position UX, point 1) + `rules/presentation.md`

## Problème constaté

La règle « strictly inward » (`rules/README.md`) est écrite mais **non verrouillée** :

1. `src/presentation/tui/app.rs:7` et `src/presentation/tui/router.rs:1` importent
   `crate::domain::pack::value_objects::TargetCli` — violation directe de l'anti-pattern
   « Importing `01_domain` directement » (`rules/presentation.md`).
2. Fuite de logique métier dans le front :
   `src/presentation/tui/router.rs:235` décide `pack.clis.first().copied().unwrap_or(TargetCli::Vibe)`
   — la règle « quelle CLI installer par défaut » vit dans le router TUI, pas dans
   l'application/domaine.
3. `src/presentation/library.rs:40` fait `std::fs::create_dir_all` — I/O direct dans
   `src/04_presentation/`.
4. Aucun test d'architecture : rien n'empêche ces dérives de réapparaître.

## Objectif

Rendre la frontière **impossible à franchir silencieusement** : les DTOs ne contiennent
plus de types du domaine, la décision de CLI par défaut migre dans la couche
application, le I/O de `library.rs` passe derrière un port, et un test d'architecture
fait échouer le build en cas de régression.

## Portée

- DTOs (`src/application/*/dto.rs`) : `clis: Vec<TargetCli>` → `Vec<String>` (ou
  newtype DTO `CliId(String)`), idem `InstalledPackDto.cli`.
- `PackService` : expose `default_cli(pack) -> String` (règle « première CLI déclarée »
  déplacée ici, testable) ; `router.rs` consomme le DTO/service uniquement.
- `src/presentation/library.rs` : `ensure_user_root` derrière une trait
  (`LibraryInitializer`) implémentée en `03_infrastructure`, ou déplacement de
  `library.rs` hors de `04_presentation` (décision au plan).
- `rules/presentation.md` : clarifier que la règle s'applique aussi aux `use` dans
  les tests co-localisés de la présentation.
- **Test d'architecture** (ex. `tests/architecture.rs`) : parcourt `src/**/*.rs`,
  assert qu'aucun fichier de `04_presentation` n'importe `domain`/`infrastructure`
  (hors `#[cfg(test)]` si la règle le permet — à trancher au plan) et qu'aucun fichier
  de `02_application` n'importe `03_infrastructure`.

## Hors scope

- Toute évolution fonctionnelle (E02–E10) — E01 est un prérequis de propreté, pas
  de feature.
- Refonte du wiring `main.rs` au-delà du déplacement du I/O de `library.rs`.

## Critères d'acceptation

- `grep -rn "crate::domain\|crate::infrastructure" src/presentation/` → 0 résultat
  (hors tests si la règle amendée le permet explicitement).
- `cargo test --test architecture` vert et intégré au gate `rules/quality.md`.
- Aucun changement de comportement visible (install/uninstall identiques).
- Gate : `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
