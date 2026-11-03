# E13 — `rapm add <dir>` / `rapm remove <name>` + kind `rules`

- **Statut** : planifié
- **Source** : consigne utilisateur — ajouter un pack directement à partir d'un
  répertoire, sans inputs multiples.
- **Décision liée** : [ADR-0001](../docs/adr/0001-refonte-cli-config-ajout-par-repertoire.md)

## Problème

`rapm pack create` demandait un item à la fois (`--skill`/`--agent`/`--command` +
`--dest <cli>=<path>` par item) : ajouter un groupe de skills/agents/commandes était
fastidieux. Il n'existait aucun moyen de supprimer un pack de la librairie.

## Objectif

```bash
rapm add <dir> [--name <n>] [--force]   # ajoute le répertoire comme pack de la librairie
rapm remove <name>                      # supprime un pack de la librairie utilisateur
```

- Le répertoire de base **peut** contenir `agents/`, `skills/`, `rules/`, `commands/`
  (chacun optionnel, au moins un requis). Chaque entrée d'un de ces répertoires devient
  un item du pack, copié sous `<kind>s/<nom>/` (ou `<kind>s/<fichier>`) dans la
  librairie ; un sous-répertoire est copié récursivement (ex. `skills/tdd/`).
- Le `pack.toml` généré recense tout le contenu : nom (`--name` ou nom du répertoire),
  version `1.0.0`, et pour chaque item une destination par profil CLI chargé, calculée
  depuis les templates du profil (`clis/*.toml`) ; `cli` = union des destinations.
- `remove <name>` supprime `<packs_dir>/<name>` ; erreur si le pack n'existe que dans
  une racine système (lecture seule).
- `rules` devient un kind d'item à part entière (`PackItemKind::Rule`) avec un template
  `rule` par profil embarqué.

## Portée

- Domaine : `PackItemKind::Rule` (`value_objects.rs`) ; templates `rule`/`rule_dir`
  dans `DestinationTemplates` (`domain/cli_profile/profile.rs`).
- Application : `src/application/pack_create/` remplacé par un service
  `add_from_dir` (scan → items → pack) + `remove` ; DTOs, port `PackWriter` réutilisé.
- Infrastructure : `FsPackWriter` adapté (écriture `<packs_dir>/<name>/`), scan du
  répertoire de base, suppression d'un pack ; `clis/*.toml` : template `rule` ajouté
  aux 7 profils embarqués.
- Présentation : `rapm add` / `rapm remove` dans `src/presentation/cli.rs` (E11).
- Tests : UT scan/génération du `pack.toml`, UT remove ; IT CLI sur tempdir
  (`add` depuis un répertoire fixture, `remove`, conflits sans `--force`).

## Hors scope

- E11 (surface CLI), E12 (config : `add` écrit dans `packs_dir`).
- Fichiers à la racine du répertoire de base (hors les 4 répertoires) : ignorés.
- Synchronisation d'un pack ajouté avec ses installations existantes.

## Critères d'acceptation

- `rapm add ./mon-pack` crée `<packs_dir>/mon-pack/pack.toml` + le contenu copié sous
  `skills/ agents/ rules/ commands/`, avec destinations générées pour chaque profil ;
  exit 0. Pack existant sans `--force` → erreur, exit 1.
- `rapm remove mon-pack` supprime le répertoire ; pack absent de la librairie
  utilisateur → erreur, exit 1.
- Un pack contenant `rules/` s'installe vers la destination `rule` du profil cible.
- Gate qualité du projet vert.
