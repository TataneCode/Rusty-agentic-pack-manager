---
name: finish-branch
description: "À utiliser quand l'implémentation est terminée et qu'il faut décider comment intégrer le travail : vérifie les tests, détermine la branche de base, présente les options (merge local ou PR), exécute le choix, nettoie l'espace de travail."
license: MIT
---

# Finish Branch — Terminer une branche

## Quand l'utiliser

Implémentation terminée, travail prêt à être intégré. Ne s'applique pas à un travail en cours.

## Le processus

### Étape 1 — Vérifier que les tests passent

Exécute la suite de tests du projet (détecte la commande : `npm test`, `pytest`, `go test`, `cargo test`...). **Lis la sortie.**

- Tout vert → passe à l'étape 2.
- Au rouge → retourne au travail (`systematic-debugging` ou `tdd`). Ne termine pas une branche qui casse.

### Étape 2 — Revue du diff

Montre le diff complet (`git diff <base>...HEAD` ou `git status` + `git diff`). Relis-le comme un relecteur (voir `code-review`, partie auto-revue). Corrige ce qui bloque.

### Étape 3 — Déterminer la branche de base

- Branche par défaut du dépôt : `main` ou `master` (vérifie : `git remote show origin` ou la config).
- Branche cible explicite (feature branch upstream) : celle-là.
- En cas de doute, demande.

Vérifie que la base est à jour (`git fetch`) et note s'il y a des conflits à prévoir.

### Étape 4 — Présenter les options

Présente à l'utilisateur, ne décide pas à sa place :

| Option | Quand | Effet |
|--------|-------|-------|
| **1. Merge local** | Branche de feature locale, base = main locale | Merge dans la base, tests re-vérifiés sur le résultat |
| **2. Push + PR** | Dépôt partagé, revue requise | Push la branche, ouvre la PR, branche le suivi revue |
| **3. Laisser tel quel** | Travail expérimental, pas prêt | Ne touche à rien, résume l'état |

### Étape 5 — Exécuter le choix

**Option 1 — Merge local :**

```bash
git checkout <base>
git pull
# merge --no-ff pour garder l'historique de la branche, ou merge classique selon la convention du dépôt
git merge <branche>
# revérifie les tests sur le résultat mergé
```

**Option 2 — Push + PR :**

```bash
git push -u origin <branche>
# ouvre la PR (gh pr create ou lien) avec : titre, résumé du diff, vérifications exécutées, points de revue
```

**Option 3 — Laisser tel quel :** résume l'état en 3 lignes (quoi, où, prochaine étape).

### Étape 6 — Nettoyer l'espace de travail

- Si le travail a utilisé un worktree : le supprimer (`git worktree remove <chemin>`).
- Supprimer les branches locales mortes si mergées.
- Vérifier qu'aucun fichier temporaire, log de debug ou instrumentation ne reste (`git status` propre).

## Si l'utilisateur demande d'abandonner le travail

Ne supprime rien sans confirmation explicite. Propose : `git checkout <base>` + branche conservée, ou `git branch -D <branche>` si vraiment jeté. Détruire du travail non commité est irréversible.

## Rationalisations interdites

| Rationalisation | Réalité |
|-----------------|---------|
| « Les tests passaient avant, pas besoin de revérifier » | Le merge change le résultat. Revérifie (étape 1 et 5). |
| « Le diff est gros, pas le temps de le relire » | C'est justement le gros diff qui mérite la relecture. |
| « Je merge directement, la revue viendra après » | Non. Revue avant merge, ou merge local explicite. |

## Check-list finale

- [ ] Suite de tests exécutée et verte (sortie lue).
- [ ] Diff relu, auto-revue faite.
- [ ] Branche de base identifiée et à jour.
- [ ] Option choisie par l'utilisateur, exécutée.
- [ ] En cas de merge : tests revérifiés sur le résultat.
- [ ] Espace de travail nettoyé, `git status` propre.
