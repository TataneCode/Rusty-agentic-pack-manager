# ADR-0001 — Refonte de la CLI, fichier de configuration et ajout de pack par répertoire

- **Statut** : accepté
- **Date** : 2026-11-03
- **Décideurs** : utilisateur du projet (consignes métier), cadrage `/challenge`

## Contexte

La CLI de `rapm` a dérivé : `rapm pack create` accepte des inputs multiples
(`--skill`/`--agent`/`--command`, chacun répété, avec une matrice `--dest <cli>=<path>`
par item et des prompts interactifs), ce qui rend l'ajout d'un groupe de skills/agents
complexe pour un humain. Les commandes réellement fondamentales — installer un pack de
la librairie, le désinstaller — sont noyées dans cette surface. Par ailleurs, la CLI
cible d'installation (vibe, gemini, opencode, claude, …) et l'emplacement de la librairie
de packs ne sont pas configurables : ils sont dérivés de l'environnement (XDG) ou du
premier CLI déclaré par le pack.

## Décision

1. **CLI réduite aux commandes fondamentales** : `rapm` (TUI), `rapm install <pack>
   [--dest <dir>] [--force]`, `rapm uninstall <pack> [--dest <dir>]`, `rapm add <dir>
   [--name <n>] [--force]`, `rapm remove <name>`, `rapm help`. Suppression de `list`,
   `show`, `pack create`, `--dry-run`, `--cli` et des prompts interactifs.
2. **Fichier de configuration TOML** lu par la CLI et le TUI, édité à la main :
   `$XDG_CONFIG_HOME/rapm/config.toml` (fallback `~/.config/rapm/config.toml`), clés
   `packs_dir` (emplacement de la librairie de packs) et `default_cli` (CLI cible par
   défaut des installations).
3. **Ajout de pack par répertoire** : `rapm add <dir>` scanne les sous-répertoires
   `agents/`, `skills/`, `rules/`, `commands/` (chacun optionnel, au moins un requis)
   et génère le `pack.toml` qui recense tout le contenu copié dans la librairie. Plus
   d'inputs multiples par item.
4. **`rapm remove <name>`** supprime un pack de la librairie utilisateur uniquement.
5. **Le TUI est conservé tel quel** (hormis la lecture de la config : librairie et CLI
   d'installation par défaut), et les **packs embarqués** (`packs/`, « starter packs »)
   restent définis dans l'application.
6. **`rules` est un nouveau kind d'item** de pack (`PackItemKind::Rule`), avec un
   template `rule` par profil CLI embarqué dans `clis/`.

## Conséquences

- `src/application/pack_create/` (service multi-inputs) est supprimé et remplacé par
  un service d'ajout par répertoire + suppression.
- Le `pack.toml` généré déclare une destination par item pour **chaque profil CLI
  chargé** (via ses templates) ; la liste `cli` du pack est l'union des destinations.
- Sans `default_cli` en config, ou si le pack ne le déclare pas, l'installation repli
  sur la **première CLI déclarée du pack** (comportement antérieur).
- `list`/`show`/`--dry-run` disparaissent de la CLI headless : les tests et IT
  associés (`tests/pack_create.rs`, `tests/cli_dry_run.rs`, parties de
  `tests/cli_headless.rs`) sont supprimés ou réécrits ; `MANUAL.md`/`README.md` à jour.
- Nouveau kind `rules` : `PackItemKind::Rule`, templates `rule` dans les 7 profils
  `clis/*.toml` et dans `DestinationTemplates`.

## Alternatives envisagées

- **Conserver `pack create` avec inputs multiples** — rejeté : c'est la source de la
  dérive dénoncée.
- **Sous-commande `rapm config set/get` + `extra_roots` + variable d'env** (brouillon
  `docs/plans/pack-library-config.md`) — rejeté pour cette itération : la config est un
  fichier édité à la main, lu des deux côtés ; ces extensions restent possibles en v2.
- **Garder `list`/`show` en lecture seule** — rejeté (scope minimal) : supprimés.
- **Générer les destinations uniquement pour le `default_cli`** — rejeté : le pack
  reste installable pour toute CLI profilée, la config ne fait que choisir la défaut.

## Points ouverts (à trancher ultérieurement, défauts actés ci-dessus)

1. Templates `rule` par profil CLI : valeurs embarquées proposées dans `clis/*.toml`.
2. Repli « première CLI déclarée » quand `default_cli` n'est pas supporté par le pack
   (au lieu d'une erreur stricte).
3. Migration des packs existants quand `packs_dir` change (v2).

## Liens

- Évolutions : [.evolutions/](../../.evolutions/index.md) — E11 (CLI simplifiée),
  E12 (config), E13 (add/remove + `rules`), E14 (ce log ADR).
- Brouillon supersédé : [docs/plans/pack-library-config.md](../plans/pack-library-config.md).
