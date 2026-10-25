---
name: tdd
description: "À utiliser pour toute fonctionnalité ou correction de bug, avant d'écrire le code d'implémentation : cycle Red-Green-Refactor avec vérification observée du RED puis du GREEN. Test d'abord, code ensuite, preuve à chaque étape."
license: MIT
---

# TDD — Test d'abord

## La loi de fer

**Pas de code d'implémentation avant un test rouge.** Si tu écris du code fonctionnel avant le test, ce n'est plus du TDD, c'est du « tester après » — moins fiable, plus coûteux.

## Le cycle Red-Green-Refactor

### RED — Écris un test rouge

1. Écris UN test qui décrit le comportement attendu.
2. Il doit être petit, nommé pour ce qu'il vérifie (`should reject expired token`).
3. Il doit échouer pour la bonne raison (fonctionnalité absente), pas pour une erreur de setup.

### VERIFY RED — Regarde-le échouer

Exécute le test. **Lis la sortie.** Confirme qu'il échoue avec le message attendu. C'est la preuve que le test teste bien quelque chose.

### GREEN — Code minimal

Écris le minimum de code pour faire passer le test. Pas d'élégance, pas de généralisation, pas de fonctionnalité voisine.

### VERIFY GREEN — Regarde-le passer

Exécute le test. **Lis la sortie.** Il passe. La `verification-gate` s'applique ici.

### REFACTOR — Nettoie

Améliore le code (noms, structure) sans changer le comportement. La suite de tests doit rester verte après chaque petit pas.

### Répète

Un cycle par comportement. Le prochain test rouge démarre le cycle suivant.

## Un test de qualité

- Teste un comportement, pas une implémentation (ne couple pas le test aux détails internes).
- Un seul concept par test ; nom explicite.
- Données de test claires (cas nominal, limite, invalide).
- Rapide et indépendant (pas d'ordre, pas d'état partagé).

## Pour un bug (TDD rétrospectif)

1. Écris un test qui **reproduit** le bug (il échoue = RED).
2. Corrige (GREEN).
3. Vérifie que le test passe et que la suite ne régresse pas.

## Rationalisations interdites

| Rationalisation | Réalité |
|-----------------|---------|
| « C'est trop petit pour un test » | C'est souvent le petit qui casse le grand. |
| « J'écris le test après, je connais le code » | Le test après confirme tes biais. |
| « Le test est trivial » | Un test trivial qui échoue puis passe prouve le fix. |
| « Pas le temps » | Le temps perdu à debugger du code non testé dépasse le temps du test. |

## Red flags — STOP et recommence

- Le code existe déjà et aucun test ne l'a vu échouer.
- Tu modifies le test pour qu'il passe au lieu de corriger le code.
- Tu écris plusieurs fonctionnalités avant de faire passer le premier test.

## Check-list de vérification

- [ ] Chaque comportement ajouté a son test écrit AVANT l'implémentation.
- [ ] J'ai vu le test échouer (sortie lue).
- [ ] J'ai vu le test passer (sortie lue).
- [ ] La suite complète passe (sortie lue).
- [ ] Le refactor garde la suite verte.

## Si on est bloqué

- Test qui ne veut pas être rouge → le comportement existe déjà peut-être ; vérifie ce que le test mesure vraiment.
- Test rouge qui ne devient pas vert → réduis le pas : code encore plus minimal, ou reviens à `systematic-debugging`.
- Trop de tests qui échouent en cascade → corrige dans l'ordre des dépendances, un à un.
