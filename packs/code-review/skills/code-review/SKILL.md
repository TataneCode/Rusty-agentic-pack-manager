---
name: code-review
description: "Revue de code rigoureuse : à utiliser pour demander une revue avant de merger (analyse du diff, gravité des remarques, points bloquants) ou pour recevoir des retours de revue (vérifier chaque remarque avant de l'implémenter, pousser back avec arguments quand c'est justifié)."
license: MIT
---

# Code Review — Revue de code

Deux usages : **demander** une revue, et **recevoir** des retours.

## Partie A — Demander une revue

### Quand

- Tâche terminée, feature majeure, avant tout merge/PR.
- La vérification locale est passée (voir `verification-gate`).

### Comment

1. **Prépare le terrain** : montre le diff (`git diff`), pas le fichier entier. Résume ce que tu as changé et pourquoi en 3–5 lignes.
2. **Auto-revue d'abord** : relis ton propre diff comme un étranger. Corrige ce que tu vois avant de demander.
3. **Guide le relecteur** : liste les points où tu veux un avis (décision d'architecture, zone risquée, compromis adopté).
4. **Demande la revue structurée** :

```markdown
## Revue demandée : [titre]

### Contexte
[2–3 lignes : quoi et pourquoi]

### Diff
[git diff ou lien]

### Vérifications faites
- [commande] → [résultat]

### Points à creuser
1. [décision 1]
2. [zone risquée 2]
```

### Ce que la revue doit produire

Chaque remarque classée par gravité :

| Gravité | Signification | Action |
|---------|---------------|--------|
| **Bloquant** | Bug, sécurité, régression, donnée perdue | Corriger avant merge |
| **Majeur** | Mauvaise pratique, dette technique, lisibilité | Corriger ou ticket explicite |
| **Mineur** | Style, nommage, commentaire | Corriger si rapide, sinon noter |
| **Question** | Choix à valider, information manquante | Répondre, pas implémenter |

Règle : un « Bloquant » non résolu = pas de merge.

## Partie B — Recevoir une revue

### La règle

**Vérifie avant d'acquiescer.** Un retour de revue n'est pas un ordre. Comprends-le, évalue-le, puis réponds.

### Le pattern de réponse

Pour chaque remarque :

1. **Lis-la en entier.** Ne réponds pas à la première ligne seulement.
2. **Vérifie le claim.** Le relecteur a-t-il raison ? Reproduis si c'est technique (lis le code, teste).
3. **Classe** : d'accord / pas d'accord / pas clair / hors scope.
4. **Réponds explicitement** : pas de « ok » muet, pas d'implémentation en silence.

###Réponses interdites

- « Bien vu, je corrige » (sans avoir vérifié que c'est vrai).
- Corriger sans lire la remarque jusqu'au bout.
- Ignorer une remarque Bloquant sans dire pourquoi.
- « Ce n'est pas grave » sans argument.

### Quand pousser back

Pousse back quand :
- Le claim est factuellement faux (preuve à l'appui : code, test, doc).
- La remarque sort du scope de la branche (propose un ticket séparé).
- Le « problème » est une préférence de style sans impact (référence la convention du dépôt si elle existe).
- Le changement casserait autre chose (explique quoi, preuve à l'appui).

Formule un pushback constructif : « Je ne suis pas d'accord parce que [preuve]. Par contre [contre-proposition]. Tu confirmes ? »

### Ordre d'implémentation des retours

1. Bloquants d'abord.
2. Majeurs ensuite.
3. Mineurs si rapides.
4. Questions : réponds, n'implémente pas.
5. Hors scope : note-les, ne les fais pas dans cette branche.

Après chaque correction : relance la vérification concernée (voir `verification-gate`), puis réponds au fil de revue point par point.

## Pièges

- **Accord performatif** : dire oui à tout pour aller vite. Chaque « oui » non tenu coûte plus cher plus tard.
- **Revue de façade** : demander une revue sans diff ni contexte.
- **Merge avec un Bloquant ouvert** : jamais.
