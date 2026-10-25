---
name: systematic-debugging
description: "À utiliser dès qu'un bug, un test en échec ou un comportement inattendu apparaît, avant de proposer le moindre correctif : enquête en 4 phases — cause racine, analyse de pattern, hypothèse testée, puis fix. Ne corrige jamais sans avoir compris."
license: MIT
---

# Systematic Debugging — Débogage systématique

## La loi de fer

**Ne corrige pas sans avoir compris.** Un fix appliqué à un symptôme est un nouveau bug en préparation. La séquence est obligatoire : cause racine d'abord, correctif ensuite.

## Les 4 phases

### Phase 1 — Enquête sur la cause racine

Objectif : comprendre pourquoi le comportement se produit, pas seulement où.

1. **Reproduis** : écris les étapes exactes qui déclenchent le problème. Lance-les. Confirme le symptôme.
2. **Lis l'erreur en entier** : message complet, stack trace, logs, code de sortie. Ne tronque pas.
3. **Inspecte l'état** : valeurs des variables au moment du crash, contenu des fichiers, sortie des commandes intermédiaires.
4. **Localise** : ajoute temporairement des logs/prints si besoin, ou lis le code du chemin d'exécution.
5. **Demande « pourquoi » 5 fois** : remonte la chaîne causale jusqu'à la cause première (pas seulement le maillon le plus proche).

Règle : à la fin de la phase 1, tu dois pouvoir dire en une phrase : « Ça casse parce que X, et X est faux à cause de Y. »

### Phase 2 — Analyse de pattern

1. Ce bug ressemble-t-il à un bug passé dans ce dépôt ? Cherche (`grep`, historique git).
2. Le pattern est-il local (une fonction) ou systémique (plusieurs endroits) ?
3. Y a-t-il d'autres endroits avec le même défaut latent ? Liste-les sans les corriger encore.

### Phase 3 — Hypothèse et test

1. Formule UNE hypothèse testable pour la cause racine.
2. Conçois le test minimal qui la confirme ou l'infirme (test de reproduction, requête, script).
3. Exécute-le. Si l'hypothèse est infirmée, retourne à la phase 1. Ne « essaie pas des trucs » au hasard.

### Phase 4 — Implémentation

1. Corrige la cause racine, pas le symptôme.
2. Le fix est minimal et chirurgical (voir `karpathy-guidelines`).
3. **Vérifie** (voir `verification-gate`) : le test de reproduction passe, la suite de tests ne régresse pas.
4. Nettoie : supprime les logs/prints temporaires, les fichiers de test jetables.
5. Si le pattern était systémique (phase 2), signale les autres sites concernés — corrige-les si c'est dans le scope, mentionne-les sinon.

## Red flags — STOP et reviens au processus

- Tu t'apprêtes à modifier du code sans avoir reproduit le bug.
- Tu as changé 3 choses et « peut-être que ça marche maintenant ».
- Tu ne peux pas expliquer la cause racine en une phrase.
- Le « fix » consiste à supprimer un message d'erreur ou à ajouter un `try/catch` qui avale l'exception.

## Signaux que l'utilisateur t'indique que tu dérapes

- « On a déjà essayé ça » → tu proposes une variation sans nouvelle hypothèse.
- « Pourquoi tu modifies encore » → tu itères sans preuve.
- « Ce n'est pas ça le problème » → tu n'as pas écouté le symptôme réel.

## Rationalisations interdites

| Rationalisation | Contre-argument |
|-----------------|-----------------|
| « C'est évident, pas besoin d'enquêter » | L'évidence est le parent du faux fix. 2 minutes d'enquête. |
| « Je vais corriger et on verra » | Non. Hypothèse d'abord, test, puis fix. |
| « Le bug est intermittent, on ne peut pas le reproduire » | Rends-le reproductible (logs, seed, environnement), ou dis-le honnêtement. |

## Si le processus ne trouve pas de cause racine

Dis-le explicitement : « Je n'ai pas trouvé la cause racine. Voici ce que j'ai écarté : [...]. Prochaine piste : [...]. » Ne fabrique pas un fix au hasard pour sauver la face.
