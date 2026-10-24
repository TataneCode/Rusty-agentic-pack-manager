# Rusty Agentic Pack Manager

TUI Rust de gestion de packs agentiques : création, déploiement et suppression d'ensembles de skills, agents et commands dans n'importe quel dépôt.

## Qu'est-ce qu'un pack agentique ?

Un **pack** est une combinaison cohérente de skills, d'agents et de commands, applicable à un CLI donné (Vibe CLI, Gemini CLI, …) et facilement déployable — et supprimable — d'un dépôt.

- **Format** : un répertoire contenant un manifeste `pack.toml` (nom, description, version, CLIs cibles, items) et les fichiers du pack (skills, agents, commands).
- **Granularité** : packs thématiques (ex. `tdd`, `review`) regroupant skills + agents + commands liés, plus packs solo.
- **Installation** : le manifeste déclare, par CLI, les destinations (ex. `.agents/skills/` pour Vibe, `.gemini/commands/` pour Gemini) ; les fichiers sont copiés sous le répertoire choisi par l'utilisateur.

## UI — split screen

- **Panneau gauche** : liste des packs, triés par CLI cible (Vibe CLI, Gemini CLI, …).
- **Panneau droit** : arborescence de fichiers de la machine hôte.

| Touche | Action |
|---|---|
| `Tab` | Basculer le focus entre les panneaux |
| `Entrée` / `Retour arrière` | Naviguer (descendre / remonter) dans la liste ou le système de fichiers |
| `A` | Appliquer un pack (focus passe sur l'explorer en mode installation) |
| `i` | Installer le pack dans le répertoire sélectionné (mode installation) |
| `r` | Créer un dossier (explorer) |
| `Espace` | Aide contextuelle (selon le panneau actif / le mode) |

En mode installation, seules la navigation, l'installation (`i`) et l'aide sont disponibles.

## Architecture

Rust (édition 2021, MSRV 1.88), TUI `ratatui` + `crossterm`. DDD 4 couches, flux strictement inward :

```
04_presentation (ui)  →  02_application  →  01_domain  ←  03_infrastructure
```

Les règles d'architecture, le design et les patterns d'actions sont repris de [rusty-console](https://github.com/TataneCode/rusty-console).

- Point d'entrée agents : [AGENTS.md](AGENTS.md), puis [CONTEXT.md](CONTEXT.md)
- Règles par couche : [rules/](rules/)
- Plans de conception : [docs/plans/](docs/plans/)

> Note : les fichiers agentiques (`AGENTS.md`, `CONTEXT.md`, `rules/`, `.agents/`, `.vibe/`, `agents/`) sont issus de rusty-console et seront adaptés à ce projet au fil de l'implémentation.

## Packs embarqués

La liste de packs initiale (dans `packs/`) est construite à partir de [docs-tech/llm-cli](https://github.com/TataneCode/docs-tech/tree/main/llm-cli) et de rusty-console, rendue agnostique des bibliothèques tierces, et ciblée **Vibe CLI** et **Gemini CLI** dans un premier temps.

## Packaging

- **RPM / DEB** : le répertoire `packs/` est embarqué dans le paquet (ex. `/usr/share/rusty-agentic-pack-manager/packs/`).
- **Windows** : une archive 7z est générée en artefact (binaire + répertoire `packs/` à côté).

## État

Projet en initialisation — voir [docs/plans/init.md](docs/plans/init.md). V1 en cours de construction.
