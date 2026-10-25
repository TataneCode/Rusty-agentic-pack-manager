---
name: work-slicing
description: "À utiliser quand un plan ou une spec existe et doit être découpé en morceaux actionnables : produit une liste de tranches verticales, chacune autonome et vérifiable, avec ses dépendances déclarées."
license: MIT
---

# Work Slicing — Découper le travail

Transforme un plan ou une spec en un ensemble de tranches actionnables. C'est l'étape entre « on sait ce qu'on construit » et « quelqu'un peut prendre un morceau ».

## Quand l'utiliser

- Un plan ou une spec existe et doit être découpé en travaux fileables.
- Une tâche est trop grosse pour une seule session et doit être partagée.
- Plusieurs personnes ou sessions vont travailler en parallèle et ont besoin de frontières claires.

## Quand ne pas l'utiliser

- Les décisions sont encore ouvertes : découper un travail non tranché produit des tranches qui seront rouvertes. Utilise d'abord `brainstorming` ou `thought-partner`.
- Le travail tient en une session : une tranche, voire aucune.
- Tu veux de l'exécution parallèle tout de suite : utilise `parallel-agents`.

## Méthode

### 1. Découper verticalement

Chaque tranche coupe un chemin étroit mais **complet** à travers toutes les couches qu'elle touche — schéma, API, UI, tests. Ce n'est PAS une tranche horizontale d'une seule couche.

- Une tranche terminée est démontrable ou vérifiable seule.
- Chaque tranche tient dans une fenêtre de contexte fraîche (quelques fichiers, pas un demi-projet).

### 2. Déclarer ce qui bloque quoi

Pour chaque tranche, liste les tranches dont elle dépend. Une tranche sans dépendance est prête. Une dépendance en cycle bloque les deux — recoupe.

### 3. Produire la sortie

Pour chaque tranche, un bloc :

```markdown
## Tranche N : [titre]
- Objectif : [ce qui marche à la fin, en une phrase]
- Fichiers : `chemin/a`, `chemin/b`
- Dépend de : Tranche M (ou « aucune — prête »)
- Vérification : `[commande]` → [résultat attendu]
- Estimation : [temps concret]
```

### 4. Classer

Donne l'ordre d'exécution : d'abord les tranches sans dépendance, puis par chaîne de dépendances.

## Règles d'arrêt

- Si découper révèle une décision non prise, arrête-toi et reviens à la phase de cadrage.
- Si une tranche grossit en cours de route, recoupe-la plutôt que de la laisser gonfler.
- Ne découpe pas au-delà du utile : 5–8 tranches pour un projet moyen, pas 30 micro-tâches.

## Contrat de sortie

- Liste numérotée de tranches, chacune avec objectif, fichiers, dépendances, vérification, estimation.
- Ordre d'exécution explicite.
- Ce qui est volontairement hors scope.

## Vérification

- Chaque tranche se lit seule : un lecteur qui la prend sans le reste comprend quoi faire et comment vérifier.
- La somme des tranches couvre le plan initial sans trou ni chevauchement.
