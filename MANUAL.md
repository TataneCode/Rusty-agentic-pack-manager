# Manuel utilisateur — rapm

`rapm` est le TUI de gestion de packs agentiques : lister, visualiser et installer des ensembles de skills, agents et commands dans un dépôt, pour un CLI cible (Vibe CLI, Gemini CLI).

## Démarrage

```bash
rapm
```

L'application démarre dans le répertoire courant. Les packs sont recherchés dans :

1. `<répertoire du binaire>/packs/`
2. `./packs` (répertoire courant)
3. `/usr/share/rusty-agentic-pack-manager/packs/` (installation système RPM/DEB)

## L'écran

Split screen :

- **Panneau gauche — Packs** : liste des packs, groupés par CLI cible.
- **Panneau droit — Explorer** : arborescence de fichiers de la machine hôte (dossiers en bleu, en premier).

La barre d'action en bas affiche les touches disponibles selon le panneau actif. Le panneau qui détient le focus a une bordure cyan.

## Touches

### Panneau Packs (focus gauche)

| Touche | Action |
|---|---|
| `Tab` | Passer le focus sur l'explorer |
| `↑`/`↓` ou `k`/`j` | Se déplacer dans la liste des packs |
| `A` | **Apply** : préparer l'installation du pack sélectionné (le focus passe sur l'explorer en mode installation) |
| `V` | **View** : ouvrir la visionneuse du pack (fichiers `.md`, `.js`, `.toml`, avec coloration syntaxique) |
| `Espace` | Aide contextuelle |
| `Ctrl-C` | Quitter |

### Panneau Explorer (focus droit)

| Touche | Action |
|---|---|
| `Tab` | Passer le focus sur les packs |
| `↑`/`↓` ou `k`/`j` | Se déplacer dans la liste |
| `Entrée` | Ouvrir le dossier sélectionné |
| `Retour arrière` | Remonter au dossier parent |
| `R` | Créer un dossier dans le répertoire courant (dialog de saisie : nom, `Entrée` valide, `Esc` annule) |
| `Espace` | Aide contextuelle |
| `Ctrl-C` | Quitter |

### Mode installation (après `A`)

Seules ces actions sont disponibles :

| Touche | Action |
|---|---|
| `↑`/`↓` ou `k`/`j` | Se déplacer dans la liste |
| `Entrée` | Ouvrir le dossier sélectionné |
| `Retour arrière` | Remonter au dossier parent |
| `I` | **Installer le pack dans le répertoire courant de l'explorer** |
| `Esc` | Annuler et revenir aux packs |
| `Espace` | Aide contextuelle |

L'installation copie chaque item du pack à sa destination déclarée dans le manifeste, **sous le répertoire choisi**. Si une destination existe déjà, l'installation est refusée (aucun écrasement silencieux) et un message d'erreur s'affiche.

### Visionneuse (après `V`)

| Touche | Action |
|---|---|
| `Tab` | Basculer entre la liste des fichiers et le contenu |
| `Entrée` | Afficher le contenu du fichier sélectionné |
| `↑`/`↓` | Défiler / changer de fichier |
| `Retour arrière` | Revenir à la liste des fichiers |
| `Esc` | Fermer la visionneuse |

## Format d'un pack

Un pack est un répertoire dans `packs/` contenant un manifeste `pack.toml` :

```toml
name = "tdd"
version = "1.0.0"
description = "TDD discipline pack"
cli = ["vibe", "gemini"]

[[items]]
kind = "skill"           # skill | agent | command
source = "skills/tdd"   # chemin dans le pack (fichier ou répertoire)
[items.destinations]
vibe = ".agents/skills/tdd"
gemini = ".gemini/skills/tdd"
```

- `name` : minuscules, chiffres, `-`, `_` ; commence par une lettre.
- `version` : semver `MAJOR.MINOR.PATCH` (suffixe `-rc.1` autorisé).
- `destinations` : chemins relatifs au répertoire cible choisi à l'installation ; les chemins absolus ou remontants (`..`) sont refusés.

## Packs embarqués

13 packs livrés : `tdd`, `code-review`, `debugging`, `security`, `planning`, `brainstorming`, `quality-gate`, `prd`, `work-slicing`, `karpathy`, `parallel-agents`, `focus-adhd`, `rusty-agnostic`.
