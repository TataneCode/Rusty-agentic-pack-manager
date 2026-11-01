# E09 — Scope `user` d'installation

- **Statut** : proposé
- **Source** : position UX — aujourd'hui « user » ne s'applique qu'à la bibliothèque
  de packs (lecture) ; il n'existe pas de scope user pour les assets déployés.

## Problème

Un skill utile pour tous les projets doit être installé projet par projet, ou copié
à la main dans `~/.agents/...` hors de tout suivi.

## Objectif

Deux scopes d'installation explicites :

- `project` (défaut) : répertoire courant, index `packs_index.toml` local —
  comportement actuel.
- `user` : racine globale configurable (ex. `$XDG_DATA_HOME/rapm/user-target/`, ou
  directement `~` avec destinations absolues résolues), index global
  `$XDG_DATA_HOME/rapm/packs_index.toml`. Les destinations du manifest restent
  relatives — elles sont résolues contre la racine user.

```bash
rapm install tdd --scope user
rapm list --scope user
```

TUI : sélecteur de scope (popup au lancement ou touche dédiée), le panel Installed
affiche le scope de chaque entrée.

## Portée

- Domaine : `InstallScope { Project, User }` value object.
- Application : résolution de la racine cible selon le scope ; index par scope.
- Infrastructure : `FsInstallIndexRepository` paramétré par racine d'index.
- CLI + TUI + MANUAL.md.

## Hors scope

- Installation user vers `~` directement (destinations absolues) — dangereux, refusé
  par la règle « relative paths only » ; le scope user reste confiné à une racine
  dédiée. À réévaluer au plan.

## Critères d'acceptation

- `--scope user` écrit sous la racine user + index global ; `list --scope user` le
  relit ; uninstall user fonctionne.
- Le scope project est inchangé (rétrocompatible).
- Gate qualité vert.
