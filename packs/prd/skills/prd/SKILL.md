---
name: prd
description: "Rédige un PRD (Product Requirements Document) optimisé pour un agent : clarification obligatoire avec l'utilisateur, recherche rapide limitée, rédaction structurée, auto-évaluation. À utiliser pour cadrer une nouvelle feature ou un produit."
license: MIT
---

# PRD — Product Requirements Document

## Phase 0 — Clarification (obligatoire)

Avant d'écrire la moindre ligne de PRD, pose ces questions à l'utilisateur :

1. **Utilisateurs cibles** : qui va s'en servir ? (développeurs, utilisateurs finaux, admins...)
2. **Problème central** : quel douleur cela résout-il ? Chiffres si disponibles.
3. **Critères de succès** : comment mesure-t-on que ça réussit ?
4. **Contraintes** : techniques, budget, délai.
5. **Contexte** : greenfield ou intégration dans un système existant ?

**Attends la réponse avant de continuer.** Si l'utilisateur dit « skip » ou donne la description en ligne, extrais ce que tu peux et note les hypothèses.

## Phase 1 — Recherche rapide (max 2 recherches)

Uniquement si le domaine est inconnu :

- 1 recherche pour le contexte domaine/marché.
- 1 recherche pour les patterns techniques éventuels.

Pas plus. 60 secondes max pour cette phase. Ne fais pas de recherche approfondie.

## Phase 2 — Rédaction du PRD

```markdown
# PRD : [Nom]

## 1. Vision
Une phrase : ce que le produit/feature accomplit.

## 2. Utilisateurs cibles
- [Profil 1] : besoin, contexte d'usage.

## 3. Problème
Quel douleur, pour qui, aujourd'hui. Fréquence et impact.

## 4. Objectifs et critères de succès
- [Objectif 1] → mesurable : [métrique]

## 5. Fonctionnalités

### Must have (MVP)
- [Fonctionnalité] : description, utilisateur, critère d'acceptation.

### Should have
- ...

### Won't have (hors scope)
- ...

## 6. Contraintes et hypothèses
- Contraintes : [technique, budget, délai]
- Hypothèses : [à valider]

## 7. Questions ouvertes
- [Question] → qui décide, quand.

## 8. Prochaines étapes
1. [Étape concrète, ex. : valider ce PRD, puis découper en plan (`writing-plans`)]
```

## Phase 2.5 — Revue adversariale (recommandée)

Relis ton propre PRD avec un œil hostile :

- Chaque « must have » est-il vraiment indispensable au MVP ?
- Chaque critère de succès est-il mesurable, pas flou ?
- Y a-t-il une fonctionnalité « au cas où » à reclasser en « won't have » ?
- Les hypothèses sont-elles testables ?

Corrige ce que la revue révèle. Marque les changements.

## Phase 3 — Auto-évaluation

Note le PRD sur 10 avec cette grille, et justifie chaque note en une ligne :

| Critère | Note /10 |
|---------|----------|
| Clarté du problème | |
| Utilisateurs bien définis | |
| Critères de succès mesurables | |
| Scope MVP réaliste | |
| Questions ouvertes identifiées | |

Si une note est < 7, corrige avant de présenter. Si tout est ≥ 7, présente.

## Phase 4 — Sauvegarde

Range le PRD dans le dépôt (proposition : `docs/prd/<date>-<sujet>.md`) et résume en 5 lignes : vision, MVP, métrique clé, contrainte principale, prochaine étape.

## Règles

- Le PRD décrit le **quoi** et le **pourquoi**, jamais le **comment** (pas de choix techniques d'implémentation).
- Un critère de succès se mesure. « Les utilisateurs aiment » n'est pas un critère.
- Le MVP est le plus petit ensemble qui prouve la valeur. Coupe sans pitié.
