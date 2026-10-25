---
name: verification-gate
description: "À utiliser avant d'affirmer qu'un travail est terminé, corrigé ou passant, avant tout commit ou merge : exige d'exécuter la commande de vérification et de constater la sortie. Aucune affirmation de succès sans preuve exécutée."
license: MIT
---

# Verification Gate — Porte de vérification

## La loi de fer

**Aucune affirmation de succès sans preuve exécutée.** « Ça devrait marcher », « le fix est en place », « les tests passent » — ce ne sont pas des preuves. La seule preuve est une commande exécutée dans cette session dont tu as lu la sortie.

## La porte

Avant de dire que quelque chose est terminé, corrigé ou vérifié, applique la fonction porte :

1. **Quelle commande prouve ce claim ?** (test, lint, build, requête, script)
2. **Exécute-la.**
3. **Lis la sortie.** Succès partiel, avertissements, sortie inattendue = pas de succès.
4. **Cite la preuve** dans ta réponse : commande + résultat constaté.

Si aucune commande ne peut prouver le claim, dis-le : « Je n'ai pas pu vérifier X, parce que Y. » Ne fais pas semblant.

## Ce qui compte comme preuve

- Sortie de test : `X passed, 0 failed` (lu, pas supposé).
- Sortie de build/lint : code 0, pas d'erreur.
- Requête exécutée : réponse observée (code HTTP, contenu).
- Fichier produit : relu après écriture.
- Reproduction d'un bug : le test de reproduction échoue AVANT le fix, passe APRÈS (preuve red-green).

Ce qui ne compte PAS : « le code est correct », « j'ai confiance », « ça a marché la dernière fois », un test que tu n'as pas lancé.

## Table de rationalisation (interdictions)

| Rationalisation | Réalité |
|-----------------|---------|
| « C'est un petit changement, pas besoin de vérifier » | Les petits changements cassent aussi. La porte s'applique à tout. |
| « J'ai vérifié dans une session précédente » | Cette session, cette preuve. Re-exécute. |
| « Les tests sont sûrement verts » | « Sûrement » n'est pas une preuve. Exécute. |
| « Le CI s'en chargera » | Le CI vérifie après coup ; toi, tu vérifies avant de claim. |

## Red flags — STOP

- Tu t'apprêtes à écrire « done », « fixed », « ça marche » sans avoir exécuté la vérification dans ce tour.
- La commande de vérification a échoué et tu envisages de dire que c'est bon quand même.
- Tu invoques une preuve que tu n'as pas sous les yeux.

## Quand s'applique la porte

- Fin de tâche, fin de bugfix, fin de feature.
- Avant tout commit, push, merge, PR.
- Avant de répondre « c'est bon » à l'utilisateur.
- Exception : la tâche est purement conversationnelle (expliquer un concept, rédiger un texte sans exécution) — pas de porte, mais ne claim aucune exécution.

## Intégration avec les autres skills

- `tdd` : la porte est le « verify RED » et le « verify GREEN ».
- `systematic-debugging` : la phase 4 (correctif) ne se termine que porte franchie.
- `finish-branch` : étape 1 = exécuter la suite de tests, lire la sortie.
