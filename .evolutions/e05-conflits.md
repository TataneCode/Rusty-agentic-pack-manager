# E05 — Stratégie de conflits à l'installation

- **Statut** : proposé
- **Source** : aujourd'hui l'install est refusé si une destination existe (pas de
  silent overwrite, `MANUAL.md`) — mais aucune alternative n'est proposée.

## Problème

Refus brutal = friction. L'utilisateur veut pouvoir choisir : garder l'existant,
écraser, ou installer à côté (suffixé).

## Objectif

Stratégie explicite, par défaut sûre :

```bash
rapm install <pack> --on-conflict refuse|skip|overwrite|suffix   # défaut: refuse
```

- `refuse` : comportement actuel (erreur, rien n'est écrit).
- `skip` : les destinations existantes sont laissées telles quelles, le plan (E04)
  les marque `skipped` ; l'index n'enregistre que ce qui a été copié.
- `overwrite` : écrase (avec backup `.bak` optionnel `--backup`).
- `suffix` : installe à côté avec un suffixe `-<n>`.

TUI : en cas de conflit à `A`, popup proposant les stratégies (défaut `refuse`).

## Portée

- Domaine : `ConflictPolicy` value object, règle appliquée dans `plan_install`.
- Application : `install(..., policy)` ; plan DTO enrichi du statut par entrée.
- Infrastructure : copie selon le plan résolu.
- CLI flag + popup TUI + MANUAL.md.

## Hors scope

- Fusion 3-way de contenu — jamais en v1.

## Critères d'acceptation

- Chaque politique produit le résultat attendu sur tempdir (IT), l'index reflète
  exactement ce qui a été écrit.
- Défaut `refuse` = comportement actuel, aucune régression.
- Gate qualité vert.
