# E06 — `rapm pack try <pack>` : sandbox jetable de test

- **Statut** : proposé
- **Source** : position UX — objectif « harness engineering qui colle au projet » :
  tester un skill/agent/command dans un environnement isolé avant adoption.

## Problème

Pour tester un pack, il faut l'installer dans le vrai projet puis nettoyer à la main.

## Objectif

```bash
rapm pack try <pack> [--cli <cli>] [--keep]
```

Installe le pack dans un répertoire temporaire (`$TMPDIR/rapm-try-<pid>-<ts>/` ou
`--scratch <dir>`), affiche le plan + les chemins écrits + la commande pour lancer
la CLI cible dans ce répertoire, et nettoie à la sortie sauf `--keep`. Variante
scriptable : exit 0 + impression du répertoire sur stdout pour un harness externe.

## Portée

- Application : `TryService` (compose `plan_install` + `install` + nettoyage),
  ou méthode dédiée sur `PackService`.
- CLI : sous-commande `pack try`.
- TUI (optionnel, v2 de l'évolution) : `T` sur un pack = try dans un sous-dossier
  `.rapm-try/` du répertoire courant, listé dans le panel Installed.
- Tests : IT CLI sur tempdir (création, contenu, nettoyage, `--keep`).

## Hors scope

- Exécution automatique de la CLI cible (hors scope : rapm ne lance pas d'autres
  processus LLM en v1).

## Critères d'acceptation

- `rapm pack try tdd` crée le bac à sable, installe, imprime le chemin, supprime à
  la fin ; `--keep` conserve.
- Aucun résidu dans le projet cible.
- Gate qualité vert.
