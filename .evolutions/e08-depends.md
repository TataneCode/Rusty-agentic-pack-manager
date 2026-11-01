# E08 — Dépendances entre packs (`depends`)

- **Statut** : proposé
- **Source** : `packs/vibe-wise-commands/pack.toml` porte « Requires the vibe-wise
  pack » en texte libre dans la description — non vérifié à l'install.

## Problème

Les dépendances inter-packs sont documentaires, pas contractuelles : installer
`vibe-wise-commands` sans `vibe-wise` produit un environnement cassé sans warning.

## Objectif

Champ optionnel dans `pack.toml` :

```toml
depends = ["vibe-wise"]
```

- Validation domaine : `depends` ⊆ noms valides, pas de cycle (graphe des packs de
  la bibliothèque), pas de self-dependency.
- Install : check dans l'index du projet cible (`packs_index.toml`) — dépendance
  absente → erreur explicite proposant `rapm install <dep>` ; option `--with-deps`
  pour installer récursivement.
- `rapm show <pack>` affiche les dépendances et leur statut (installé ou non).

## Portée

- `src/domain/pack/` : value object `PackDependencies`, règle de validation (cycle
  détecté sur le graphe chargé).
- `src/application/pack/service.rs` : résolution + check à l'install.
- Loader `src/infrastructure/pack/loader.rs` : parsing du champ.
- Tests : UT domaine (cycles), IT install avec dépendance manquante/présente.

## Hors scope

- Versions de dépendances (`depends = [{ name = "x", version = ">=1.2" }]`) — v2.
- Résolution depuis un registre distant — hors scope (pas de réseau en v1).

## Critères d'acceptation

- Install d'un pack avec dépendance non installée → erreur claire, exit 1 (CLI) /
  popup (TUI), aucun fichier copié.
- `--with-deps` installe la chaîne dans le bon ordre.
- Gate qualité vert.
