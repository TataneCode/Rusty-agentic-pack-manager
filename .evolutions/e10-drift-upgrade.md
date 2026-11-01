# E10 — Détection de drift + upgrade

- **Statut** : proposé
- **Source** : `packs_index.toml` enregistre ce qui a été copié
  (`src/infrastructure/pack/index.rs`) mais rien ne compare ensuite disque / index /
  version du pack.

## Problème

Après install, on ne sait pas si les fichiers ont été modifiés à la main, supprimés,
ou si une version plus récente du pack existe dans la bibliothèque.

## Objectif

```bash
rapm status [--dest <dir>]     # par pack installé : ok / modifié / supprimé / outdated
rapm upgrade <pack> [--dest <dir>]
```

- `status` : pour chaque entrée de l'index, compare les chemins copiés au disque
  (présence + hash optionnel v2) et la version installée à la version du pack en
  bibliothèque.
- `upgrade` : uninstall propre (index) puis réinstall de la version courante.
- TUI : badge/coloration dans le panel Installed (`ok` / `drift` / `outdated`),
  touche `G` = upgrade.

## Portée

- Domaine : `DriftStatus` enum, règle de comparaison (index vs disque).
- Application : `PackService::status(dest)` ; `upgrade` = uninstall + install.
- Infrastructure : lecture disque des chemins indexés.
- CLI + TUI + tests IT.

## Hors scope

- Hash de contenu (v2) : v1 = présence + taille + version.
- Upgrade automatique à l'ouverture du TUI — jamais silencieux.

## Critères d'acceptation

- Fichier supprimé à la main → `status` le marque `supprimé` ; `upgrade` restaure.
- Pack upgradé en bibliothèque (version bump) → `outdated` ; `upgrade` migre et
  met à jour l'index.
- Gate qualité vert.
