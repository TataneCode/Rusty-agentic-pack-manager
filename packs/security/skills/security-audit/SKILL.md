---
name: security-audit
description: "Audit de sécurité d'un codebase : scan OWASP Top 10, recherche de secrets, revue des dépendances, des configurations et de l'authentification. Produit un rapport priorisé (critique/majeur/mineur) avec preuves et correctifs proposés."
license: MIT
---

# Security Audit — Audit de sécurité

Audit mono-agent d'un codebase, sans orchestrateur externe. Deux niveaux : **scan rapide** (OWASP ciblé) ou **audit complet** (tout le dépôt).

## Déroulement

### 1. Périmètre

Confirme ce qui est audité : dépôt entier, module, ou diff récent. Note les exclusions (vendor, tests, fixtures).

### 2. Scan systématique

Passe en revue, dans cet ordre :

**A. Secrets et credentials**
- Recherche de clés, tokens, mots de passe en dur : `grep -riE "(api[_-]?key|secret|password|token|private[_-]?key)" --include="*.py" --include="*.js" --include="*.ts" --include="*.env*" .`
- Fichiers `.env`, credentials, certificats commités ?
- Secrets dans l'historique git ?

**B. OWASP Top 10 (les plus fréquents d'abord)**
1. **Injection** (SQL, NoSQL, OS, LDAP) : recherche de concaténation de requêtes, `eval`, `exec`, templates non échappés.
2. **Auth cassée** : gestion de session, hachage des mots de passe (bcrypt/argon2, pas MD5/SHA1), vérification des tokens.
3. **Exposition de données sensibles** : logs de données perso, réponses API trop bavardes, données en clair.
4. **XXE / désérialisation** : parsers XML, `pickle`, désérialisation non sûre.
5. **Contrôle d'accès** : routes sans auth, IDOR (référence directe non vérifiée), élévation de privilège.
6. **Mauvaise config sécurité** : CORS `*`, headers manquants, mode debug, ports exposés.
7. **XSS** : rendu de HTML/JS non échappé, `innerHTML`, templates dangereux.
8. **CSRF** : mutations sans token CSRF, SameSite mal configuré.
9. **Composants vulnérables** : dépendances connues vulnérables (`npm audit`, `pip-audit`, `cargo audit`, `govulncheck`).
10. **Journalisation insuffisante** : actions sensibles non tracées, logs de secrets.

**C. Fichiers sensibles**
- Permissions, clés privées, certificats dans le dépôt.
- Configurations de déploiement (CI, Docker, cloud) : secrets, privilèges excessifs.

### 3. Rapport

Structure du rapport :

```markdown
# Audit de sécurité — [périmètre] — [date]

## Synthèse
- Critiques : X · Majeurs : Y · Mineurs : Z
- Verdict global : [une phrase]

## Findings

### CRITIQUE — [titre]
- Où : `fichier:ligne`
- Preuve : [extrait de code ou sortie de commande]
- Impact : [ce qu'un attaquant peut faire]
- Correctif : [quoi changer, extrait proposé]

### MAJEUR — [titre]
...

### MINEUR — [titre]
...

## Hors périmètre / non vérifié
- [ex. : « historique git non audité », « tests exclus »]
```

### 4. Règles de rapport

- **Chaque finding a une preuve** : fichier:ligne + extrait, ou sortie de commande. Pas de « il y a peut-être un risque ».
- **Gravité honnête** : ne gonfle pas, ne minimise pas. Un secret commité = CRITIQUE, pas MAJEUR.
- **Correctif concret** : chaque finding critique/majeur a un extrait de code corrigé proposé.
- **Pas de fix dans ce skill** : l'audit produit un rapport. Les corrections se font dans une branche dédiée, avec `tdd` si pertinent.

## Check-list de fin

- [ ] Secrets/credentials scannés.
- [ ] OWASP Top 10 couvert (ou exclusions notées).
- [ ] Dépendances auditées (commande exécutée, sortie lue).
- [ ] Configs de déploiement revues.
- [ ] Chaque finding a preuve + gravité + correctif.
- [ ] Synthèse avec verdict global.

## Limites à déclarer

- Un audit statique ne remplace pas un test d'intrusion.
- Les findings « non vérifiés » sont listés comme tels.
- La gravité reflète l'impact potentiel, pas la probabilité d'exploitation.
