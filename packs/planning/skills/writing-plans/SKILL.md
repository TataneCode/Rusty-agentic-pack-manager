---
name: writing-plans
description: "À utiliser quand une spec ou des exigences existent pour une tâche en plusieurs étapes, avant de toucher au code : rédige un plan d'implémentation structuré, en tâches numérotées, chaque étape avec sa commande de vérification."
license: MIT
---

# Writing Plans — Rédiger un plan

## Quand l'utiliser

- Tâche en plusieurs étapes (plus de ~3 actions non triviales).
- Feature, migration, refactoring multi-fichiers.
- Quand l'utilisateur demande explicitement un plan.

Ne pas utiliser pour : une correction en un fichier, une question, une recherche.

## Structure du plan

```markdown
# Plan : [Nom de la feature]

## Objectif
Une phrase : ce que le lecteur pourra faire après.

## Prérequis
- Outil/versions/dépendances nécessaires.

## Tâches

### Tâche 1 : [Composant]
- Fichiers touchés : `chemin/vers/fichier`
- Étapes :
  1. [action]
  2. [action]
- Vérifie : `[commande]` → [résultat attendu]

### Tâche 2 : [Composant]
...

## Ordre et dépendances
Tâche 2 dépend de la tâche 1, etc.

## Hors scope
Ce que ce plan ne fait PAS (pour couper court aux dérives).

## Revue
Points de vigilance pour la revue de code.
```

## Règles de taille

- **Une tâche = un composant** (un fichier, un module, une responsabilité).
- **Une étape = une action vérifiable.** Si une étape ne peut pas être vérifiée, elle est trop grosse — découpe.
- Chaque étape se termine par « Vérifie : [commande] → [attendu] » (voir `verification-gate`).
- Pas de « et puis » en cascade dans une étape : c'est deux étapes.

## Granularité

- Trop gros (« implémenter l'auth ») → personne ne peut reprendre, tout est dans une tête.
- Trop fin (« créer le fichier X ») → bruit, pas de plan.
- Juste bien : une tâche tient en quelques minutes à une heure, se lit seule, se vérifie seule.

## Avant d'écrire le plan

1. Vérifie que la demande est claire. Si un choix structurant est ouvert (approche A ou B), pose la question ou présente les 2 options avec recommandation — n'écris pas un plan sur une fondation floue.
2. Lis le code concerné (ne planifie pas dans le vide).
3. Liste les contraintes globales (style du dépôt, pas de nouvelle dépendance, compatibilité...).

## Après le plan

- Fais-le valider par l'utilisateur avant d'exécuter.
- Range le plan dans le dépôt si le projet le prévoit (ex. `docs/plans/<date>-<sujet>.md`).
- Exécute tâche par tâche, en cochant au fur et à mesure. Ne saute pas de vérification.

## Auto-revue du plan (avant de le présenter)

- [ ] Chaque étape a sa vérification.
- [ ] Chaque tâche tient seule (fichiers nommés).
- [ ] L'ordre respecte les dépendances.
- [ ] Le hors scope est explicite.
- [ ] Aucun « on verra bien en chemin ».
