---
name: brainstorming
description: "À utiliser avant tout travail créatif — nouvelle feature, composant, fonctionnalité, modification de comportement, ou planification de quoi que ce soit : cadre d'abord le besoin réel avec l'utilisateur, avant de proposer la moindre solution."
license: MIT
---

# Brainstorming — Cadrer avant de construire

## La règle

**Ne propose pas de solution avant d'avoir compris le problème.** La séquence est : comprendre → questionner → rejouer → cadrer → seulement ensuite, concevoir.

## Phase 1 — Ce que tu sais réellement

Avant de poser des questions, vérifie ce que tu sais déjà :

1. Lis le code/la doc/le contexte concerné. Ne demande pas ce que le dépôt répond.
2. Note ce qui est connu, ce qui est incertain, ce qui est une contrainte.

## Phase 2 — Le premier message

Ton premier message pose des questions, pas de solution. Cible :

- **Le besoin réel** : pourquoi maintenant ? Quel problème cela résout-il, pour qui ?
- **Les contraintes** : budget de temps, compatibilité, conventions du dépôt, hors scope.
- **Les critères de succès** : comment saura-t-on que c'est réussi ?
- **Les non-objectifs** : ce qu'on ne fera PAS, explicitement.

Pose les questions en petit nombre, groupées. Pas 15 questions d'un coup.

## Phase 3 — La conversation

- Une question à la fois par tour si les réponses sont denses.
- Reformule ce que tu as compris pour valider (« Si je résume : ... C'est bien ça ? »).
- Pousse back quand une idée contredit une contrainte ou une bonne pratique — avec des arguments, pas un avis.
- Si l'utilisateur dérive vers l'implémentation, ramène au besoin : « On verra le comment après le quoi. »

## Phase 4 — Rejouer (playback)

Avant de passer à la conception, rejoue le cadrage :

```markdown
## Ce qu'on construit
[Une phrase]

## Pour qui et pourquoi
[Une phrase]

## Critères de succès
- [critère vérifiable 1]
- [critère vérifiable 2]

## Hors scope
- [ce qu'on ne fait pas]

## Contraintes
- [contrainte 1]
```

Fais valider ce cadrage. Ne passe à la suite que sur un « oui » explicite (ou une correction que tu intègres).

## Phase 5 — Dimensionner le travail

Estime en unités concrètes (minutes / heures / jours), pas « un peu de travail ». Dis ce qui tient dans une session et ce qui dépasse.

## Phase 6 — Le design écrit

Seulement maintenant, propose la solution :

1. **Approche retenue** + **approches écartées** (et pourquoi, en une ligne chacune).
2. **Architecture/forme** : fichiers, responsabilités, flux de données.
3. **Plan de vérification** : quoi exécuter pour prouver que ça marche.
4. **Risques** : ce qui peut coincer, et le plan B.

Termine par : « Valide ce design, ou dis-moi ce qui coince. »

## Red flags

- Tu as proposé une solution dans ton premier message.
- Tu n'as pas lu le code existant avant de questionner.
- Le cadrage n'a pas été validé et tu conçois quand même.
- L'utilisateur dit « peu importe, fais au plus simple » → ce n'est pas un blanc-seing, c'est un signal pour réduire le scope, pas pour sauter le cadrage.

## Rationalisations interdites

| Rationalisation | Réalité |
|-----------------|---------|
| « Le besoin est évident » | L'évidence de l'utilisateur n'est pas la tienne. 2 questions confirment. |
| « Brainstormer ralentit » | Un mauvais départ coûte 10× un bon cadrage. |
| « Je propose, il corrigera » | Il corrigera après coup, sur du code écrit. |
