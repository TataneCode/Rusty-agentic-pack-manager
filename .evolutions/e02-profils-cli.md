# E02 — Profils de CLI externes (registry de layouts)

- **Statut** : proposé
- **Source** : position UX, question 4 — `TargetCli` est un enum fermé à 2 variantes
  (`src/domain/pack/value_objects.rs:8`) ; chaque nouvelle CLI = recompile + mise à
  jour de chaque pack.

## Problème

La connaissance « où va un skill/agent/command pour la CLI X » est codée dans
chaque `pack.toml` (destinations explicites par CLI). Ajouter Claude Code, Cursor,
Copilot… coûte une variante d'enum et N manifests.

## Objectif

Externaliser les layouts par CLI dans des **profils chargeables** (TOML), shipés
avec rapm et surchargeables par l'utilisateur (user library, même ordre de priorité
que les packs) :

```toml
# clis/claude.toml
[cli]
name = "claude"
label = "Claude Code"
[defaults]
skill   = ".claude/skills/{name}/SKILL.md"
agent   = ".claude/agents/{name}.md"
command = ".claude/commands/{name}.md"
```

- Un pack peut alors déclarer `cli = ["claude"]` avec des items **sans destinations
  explicites** : elles sont calculées depuis le profil (`{name}` = nom de l'item).
- Les destinations explicites du manifest restent possibles (override, rétrocompatible
  avec les 16 packs existants).
- Domaine : `TargetCli` enum → newtype `CliId(String)` validée contre le registry des
  profils chargés ; l'enum actuel reste comme liste des CLIs « connues » pour
  l'affichage.

## Portée

- Nouveau bounded context `cli_profile` : domaine (validation, calcul de
  destination par défaut), application (service de résolution), infrastructure
  (loader TOML des profils, racines = user library + ship + système).
- Migration douce : `pack.toml` sans destinations pour une CLI profilée fonctionne ;
  avec destinations → inchangé.
- `TargetCliMap` : accepter une CLI absente de l'enum si un profil existe.

## Hors scope

- Profils auto-détectés depuis le binaire installé (`which claude`) — v2.
- Standard Agent Skills comme fallback universel — la couche `.agents/skills/` est
  déjà couverte par le profil vibe ; à généraliser au plan.

## Critères d'acceptation

- Un pack déclaré pour une CLI ayant un profil mais sans enum recompilé s'installe.
- Un profil utilisateur override le profil shipé de même nom.
- Les 16 packs existants continuent de s'installer sans modification.
- Gate qualité vert.
