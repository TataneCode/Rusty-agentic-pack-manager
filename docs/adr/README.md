# ADR — Architecture Decision Log

Registre des **décisions d'architecture** de `rapm` : une fiche par décision majeure,
immuable une fois acceptée. Les évolutions opérationnelles (fonctionnalités) vivent
dans [.evolutions/](../../.evolutions/index.md) ; une évolution à impact d'architecture
y est référencée et inversement.

## Conventions

- **Emplacement** : un fichier par décision ici, nommé `NNNN-titre-slug.md`
  (`NNNN` = numéro sur 4 chiffres, **immuable** : on ne renumérote jamais, une décision
  annulée garde son numéro).
- **Statuts** : `proposé` → `accepté` / `rejeté` ; `supersédé par ADR-NNNN` quand une
  décision ultérieure la remplace (le texte original reste).
- **Format** d'une fiche :

  ```markdown
  # ADR-NNNN — <titre>

  - **Statut** : accepté
  - **Date** : AAAA-MM-JJ
  - **Décideurs** : <qui>

  ## Contexte
  ## Décision
  ## Conséquences
  ## Alternatives envisagées
  ## Liens
  ```

- **Quand écrire un ADR** : toute décision qui change la surface de la CLI/TUI, le
  format des packs ou de la config, l'emplacement des données, ou les frontières entre
  couches. Les choix internes d'un plan (`docs/plans/`) n'en sont pas un.
- Une ADR acceptée ne se réécrit pas : on écrit une nouvelle ADR qui la supersède.

## Index

| ADR | Titre | Statut |
|---|---|---|
| [ADR-0001](0001-refonte-cli-config-ajout-par-repertoire.md) | Refonte de la CLI, fichier de configuration et ajout de pack par répertoire | accepté |
