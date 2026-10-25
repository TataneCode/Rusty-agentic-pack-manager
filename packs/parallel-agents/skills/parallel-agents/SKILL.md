---
name: parallel-agents
description: "À utiliser face à 2 tâches indépendantes ou plus, sans état partagé ni dépendance séquentielle : découpe en domaines focalisés, dispatch en parallèle via les sous-agents disponibles, intègre les résultats."
license: MIT
---

# Parallel Agents — Agents parallèles

## Quand l'utiliser

- 2 tâches ou plus, **indépendantes** : pas d'état partagé, pas de dépendance séquentielle, pas de conflit sur les mêmes fichiers.
- Chaque tâche est assez grosse pour justifier un agent dédié (pas 2 lignes de changement).

## Quand ne PAS l'utiliser

- Les tâches se marchent dessus (mêmes fichiers, même module).
- La tâche 2 dépend du résultat de la tâche 1.
- Le travail tient en quelques minutes en séquentiel — le parallélisme ajoute du coût de coordination.

## Le patron

### 1. Identifier les domaines indépendants

Pour chaque tâche candidate, vérifie :

- Fichiers disjoints ?
- Pas d'appel de fonction entre les deux ?
- Même résultat attendu si exécutées dans n'importe quel ordre ?

Si un doute subsiste sur l'un des trois points, ce n'est pas indépendant.

### 2. Créer des tâches focalisées

Chaque sous-agent reçoit un prompt autonome :

```markdown
## Tâche : [titre]

### Contexte
[3–5 lignes : le projet, ce qui existe déjà, les conventions]

### Objectif
[Une phrase : ce qui doit être vrai à la fin]

### Périmètre
- Fichiers à toucher : [liste]
- Hors périmètre : [ce qu'il ne faut PAS toucher — les autres tâches parallèles travaillent ailleurs]

### Étapes attendues
1. ...
2. ...

### Vérification
`[commande]` → [résultat attendu]

### Rendu
Résume : fichiers modifiés, décisions prises, vérifications exécutées (commandes + sorties), points ouverts.
```

Règle d'or du prompt : le sous-agent ne doit pas avoir besoin de savoir ce que font les autres tâches, seulement ce qu'il ne doit pas toucher.

### 3. Dispatcher en parallèle

- Lance les sous-agents en parallèle (un appel de sous-agent par tâche, même tour).
- Ne fais pas le travail toi-même en attendant : ton rôle est coordination + intégration.

### 4. Revue et intégration

À la réception de chaque rapport :

1. **Vérifie les claims** : les commandes de vérification ont-elles vraiment été exécutées ? Relis les sorties citées (voir `verification-gate`).
2. **Vérifie le périmètre** : le sous-agent n'a-t-il pas touché aux fichiers des autres tâches ?
3. **Intègre** : résous les conflits de merge si chevauchement il y a (normalement aucun).
4. **Vérifie globalement** : exécute la suite de tests du projet entier, pas seulement ceux d'une tâche.

### 5. Rendu final

```markdown
## Résultat du dispatch parallèle
- Tâche A : [résultat] — vérifié : [commande]
- Tâche B : [résultat] — vérifié : [commande]
- Intégration : [conflits résolus ou « aucun »]
- Vérification globale : [commande] → [résultat]
```

## Erreurs classiques

| Erreur | Conséquence | Fix |
|--------|-------------|-----|
| Tâches « indépendantes » qui touchent au même fichier | Conflits de merge, régression | Redécouper les frontières avant de lancer |
| Prompt de sous-agent trop vague | Résultat inutilisable | Contexte + périmètre + vérification obligatoires |
| Accepter le rapport sans vérifier | Bug passé en production | Relire les sorties, lancer la vérif globale |
| Dispatcher 5 micro-tâches | Coût de coordination > gain | Garder en séquentiel sous ~5 min |

## Adaptation selon le CLI

- **Vibe CLI** : sous-agents disponibles via la config `~/.vibe/agents/` (voir le dossier `agents/` de ce dépôt).
- **Gemini CLI** : pas de sous-agents déclarés ; le parallélisme se fait via plusieurs sessions CLI dans des worktrees git séparés, ou en séquentiel si un seul agent.
- Toujours vérifier ce que le CLI courant expose avant de promettre du parallélisme.
