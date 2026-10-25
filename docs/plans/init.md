# Plan : rusty-agentic-pack-manager — socle v1 (init)

> Statut : **approuvé** (challenge + plan validés par l'utilisateur). Source : skills `/challenge` puis `/plan`. Les décisions actées par l'utilisateur sont notées ✅.

## Objectif

Livrer un TUI Rust (split-screen) qui permet de parcourir une liste de packs agentiques embarqués (triés par CLI), de naviguer le filesystem hôte, et d'installer le contenu d'un pack dans le répertoire choisi — en reprenant l'architecture DDD 4 couches, le design (thème, widgets, triade presenter/view/actions, event loop) et les patterns d'actions de rusty-console.

## Contexte de cadrage (challenge)

- Reformulation : TUI de gestion du cycle de vie de packs agentiques (skills/agents/commands par CLI cible), déployables et supprimables dans n'importe quel dépôt.
- Décisions utilisateur :
  - ✅ **Pack = répertoire + `pack.toml`** (archives 7z/tar en v2).
  - ✅ **Granularité thématique (option B)** : packs regroupant skills + agents + commands liés (ex. `tdd`), plus packs solo ; environ 6-8 packs + solos.
  - ✅ **Flow apply** : `A` sur un pack → focus sur le panel explorer en mode installation ; seules actions possibles : navigation (Entrée/Retour arrière), `i` = installer dans le répertoire sélectionné, Espace = aide contextuelle.
- Scope creep écarté en v1 : rename (`m`), delete (`d`/`D`) sur le FS hôte ; undeploy complet (mitigation : copier le manifeste dans la cible à l'install pour le rendre possible en v2).

## Décisions d'architecture

- **D1 : Application synchrone, pas de tokio/bollard** — le périmètre v1 est purement local (FS + TUI) ; on ne reprend pas tokio, bollard, async-trait, futures-util, chrono, base64. `std::fs` + event loop crossterm suffisent (pattern `EventHandler` de rusty-console conservé, sans async).
- **D2 : Pack = répertoire + `pack.toml`** ✅ — dépendances `toml` + `serde`. Schéma du manifeste : `name`, `description`, `version`, `cli` (liste cible : `vibe`/`gemini`), `[[items]]` (kind `skill|agent|command`, source locale, destination par CLI : ex. `.agents/skills/<name>/` pour vibe, `.gemini/commands/` pour gemini).
- **D3 : Découverte des packs sur disque** — recherche dans `<dir_du_binaire>/packs/` puis `/usr/share/rusty-agentic-pack-manager/packs/`. Rend le packaging trivial (RPM/DEB embarquent `packs/` comme asset ; le 7z Windows = binaire + `packs/` à côté).
- **D4 : Domain `pack`** — entité `Pack` (identité = nom + version), value objects `PackName`, `PackVersion`, `TargetCli` (enum extensible), `PackItemKind` ; erreurs dédiées dans `01_domain/error.rs`. La **planification d'installation** (calcul des copies source→destination, détection de collisions) est une règle du domain, testable sans FS.
- **D9 : Visionneuse de pack (`v`, panel packs uniquement)** ✅ — `v` sur un pack sélectionné ouvre un écran de visualisation des fichiers du pack : `.md`, `.js`, `.toml` (tout fichier texte au sens large ; les fichiers non textuels sont refusés proprement). La visionneuse reprend le pattern « écran de logs » de rusty-console : plein écran, navigation (flèches, PgUp/PgDn, Home/End), retour par `Esc` à l'écran split-screen, `Tab` bascule entre la liste des fichiers du pack et l'aperçu du contenu. **Coloration syntaxique** via `syntect` (lecteurs de syntaxe `.md`, `.js`, `.toml` fournis par le crate) ; rendu adapté aux 16 couleurs du terminal via le thème de rusty-console. Nouvelle area application `pack_view` : trait `PackContentRepository` (list, read), DTO `PackFile { path, kind, content }`, adapter en `03_infrastructure/pack/`. Dépendance ajoutée : `syntect` (justifiée : coloration syntaxique exigée ; features par défaut sans onig — utiliser les regex du std si disponibles, sinon `default-onig`).
- **D5 : Deux areas application** — `pack` (service de listage/chargement, trait `PackRepository`) et `explorer` (service de navigation FS, traits `ExplorerRepository` : list, create_dir ; DTO `FileNode`). Adapters en `03_infrastructure/filesystem/` (std::fs) et `03_infrastructure/pack/` (lecture `pack.toml`). Frontières mockées avec `mockall` en tests, comme rusty-console.
- **D6 : UI = un seul écran à deux panneaux** — état `Focus::Packs | Focus::Explorer` basculé par Tab. La flow « apply » introduit un mode `Installing { pack }` qui force `Focus::Explorer` et restreint les actions à : navigation, `i` = installer, Espace = aide. Triade presenter/view/actions par panneau + `common/` repris de rusty-console (`theme`, `keys`, `action_bar`, aide en popup contextuel, `confirm`). `App`/`router.rs` séquencent les `AppAction`, pattern identique à rusty-console.
- **D7 : Touches v1** — Tab (focus), Entrée/Retour (navigation des deux côtés), `A` (apply → mode install), `i` (installer ici), `r` (créer dossier), `v` (visionneuse de pack, panel packs uniquement), Espace (aide contextuelle selon panneau/mode). `m`/`d`/`D` reportés v2.
- **D8 : Contenu des packs initiaux — granularité thématique** ✅, uniquement vibe + gemini, agnostiques des libs tierces. Packs prévus : `tdd`, `review`, `planning`, `debugging`, `security`, `architecture`, `quality-gate`, plus packs solo (`prd`, `work-slicing`, `thought-partner`, `brainstorming`, `karpathy-guidelines`, `parallel-agents`, `i-have-adhd`) depuis docs-tech/llm-cli ; pack `rusty-agnostic` depuis les skills/agents/commands de rusty-console. Composition exacte figée à l'étape 10.

## Étapes

1. **Squelette** — `Cargo.toml` (déps : ratatui 0.30, crossterm 0.29, thiserror, anyhow, serde, toml ; dev : mockall), `src/lib.rs`, `src/main.rs`, modules des 4 couches vides — *vérif : `cargo check`*
2. **Domain pack** — `01_domain/pack/{entity,value_objects}.rs`, `01_domain/error.rs` + tests unitaires (validation manifeste, plan d'installation, collision) — *vérif : `cargo test --lib pack`*
3. **Application** — `02_application/{pack,explorer}/` (traits, DTO, services, mappers) + tests avec mocks `mockall` — *vérif : `cargo test --lib`*
4. **Infrastructure** — `03_infrastructure/pack/` (loader `pack.toml`) et `03_infrastructure/filesystem/` (explorer, create_dir) + tests sur fichiers temporaires — *vérif : `cargo test --lib`*
5. **Socle TUI** — `04_presentation/tui/{app,event,router}.rs` + `common/` (theme, keys, action_bar) repris de rusty-console ; split-screen avec deux panneaux, Tab bascule le focus, rendu vide — *vérif : `cargo test --lib presentation`*
6. **Panneau packs** — triade `packs/{presenter,view,actions}.rs` : liste groupée par CLI, sélection, `A` passe en mode install — tests TestBackend — *vérif : `cargo test packs`*
7. **Panneau explorer** — triade `explorer/{presenter,view,actions}.rs` : navigation Entrée/Retour, `r` (créer dossier, dialog input repris de rusty-console), mode install restreint + `i` installe (confirm dialog) — tests TestBackend — *vérif : `cargo test explorer`*
8. **Aide contextuelle** — popup Espace selon focus/mode — test TestBackend — *vérif : `cargo test help`*
9. **Visionneuse de pack (`v`)** — application `pack_view` (trait + DTO + service + tests mockall) ; infrastructure `03_infrastructure/pack/content.rs` (lecture fichiers du pack) ; presentation `04_presentation/tui/viewer/` (triade presenter/view/actions : liste fichiers ↔ aperçu, coloration `syntect`, navigation logs-like) + tests TestBackend — *vérif : `cargo test viewer`*
10. **Tests d'intégration** — `tests/pack_install.rs` (pack réel → répertoire temp : fichiers copiés aux bonnes destinations), `tests/split_screen_ui.rs` (focus Tab, rendu des panneaux) — *vérif : `cargo test`*
11. **Données : rédaction des packs** — `packs/<name>/pack.toml` + contenus depuis docs-tech/llm-cli et rusty-console, adaptés vibe/gemini, agnostiques — *vérif : `cargo test --test pack_install` + revue manuelle des manifestes*
12. **Packaging** — `cargo-generate-rpm` (assets : binaire + `packs/`), `cargo-deb`, artefact 7z Windows (binaire + `packs/`) dans CI — *vérif : `cargo generate-rpm` local*
13. **Docs** — `README.md` (fait), `MANUAL.md` (touches), `CONTEXT.md`/`AGENTS.md` adaptés à ce projet, plan archivé dans `docs/plans/` (ce fichier) — *vérif : `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`*

## Risques et mitigations

- **Chemin d'installation gemini exact** (`.gemini/commands/` à confirmer) → manifeste par pack = source de vérité modifiable sans recompiler ; vérification à l'étape 10.
- **Patterns de rusty-console à dé-asyncifier** (tokio supprimé) → mitigé en gardant l'event loop synchrone simple dès l'étape 5.
- **`toml` crate vs MSRV 1.88** → vérifier la version résolue à l'étape 1, épingler si nécessaire.
- **7z Windows en CI** → nécessite `7z` dans le runner ; fallback : archive zip si l'outil manque.
- **`syntect` vs MSRV 1.88 / taille binaire** → `syntect` entraîne des dumps de syntaxe embarqués (≈2-3 Mo) ; si le MSRV ou la taille posent problème, fallback : coloration lexicale maison (regex std) limitée à `.md`/`.js`/`.toml`, décidée à l'étape 9.
- **Coloration 16 couleurs terminal** → `syntect` sort des couleurs 24-bits ; mapping vers la palette du thème ; test TestBackend sur des spans stylés.

## Hors périmètre (v1)

- Rename (`m`), delete (`d`/`D`) sur le FS hôte — v2
- Undeploy/suppression d'un pack appliqué — v2 (le manifeste copié dans la cible à l'install le rendra possible)
- Packs aux formats 7z/tar — v2
- CLIs autres que vibe et gemini — structure prête (enum `TargetCli`), contenu non fourni
