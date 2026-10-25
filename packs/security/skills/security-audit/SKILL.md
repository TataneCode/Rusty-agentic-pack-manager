---
name: security-audit
description: "Security audit of a codebase: OWASP Top 10 scan, secret hunting, dependency, configuration and authentication review. Produces a prioritized report (critical/major/minor) with evidence and proposed fixes."
license: MIT
---

# Security Audit — Security audit

Single-agent audit of a codebase, without external orchestrator. Two levels: **quick scan** (targeted OWASP) or **full audit** (whole repository).

## Process

### 1. Scope

Confirm what is audited: whole repository, module, or recent diff. Note exclusions (vendor, tests, fixtures).

### 2. Systematic scan

Review, in this order:

**A. Secrets and credentials**
- Search for hard-coded keys, tokens, passwords: `grep -riE "(api[_-]?key|secret|password|token|private[_-]?key)" --include="*.py" --include="*.js" --include="*.ts" --include="*.env*" .`
- `.env` files, credentials, committed certificates?
- Secrets in the git history?

**B. OWASP Top 10 (most frequent first)**
1. **Injection** (SQL, NoSQL, OS, LDAP): search for query concatenation, `eval`, `exec`, unescaped templates.
2. **Broken authentication**: session handling, password hashing (bcrypt/argon2, not MD5/SHA1), token verification.
3. **Sensitive data exposure**: personal data in logs, overly verbose API responses, plaintext data.
4. **XXE / deserialization**: XML parsers, `pickle`, unsafe deserialization.
5. **Access control**: unauthenticated routes, IDOR (unverified direct reference), privilege escalation.
6. **Security misconfiguration**: CORS `*`, missing headers, debug mode, exposed ports.
7. **XSS**: unescaped HTML/JS rendering, `innerHTML`, dangerous templates.
8. **CSRF**: mutations without CSRF token, misconfigured SameSite.
9. **Vulnerable components**: known vulnerable dependencies (`npm audit`, `pip-audit`, `cargo audit`, `govulncheck`).
10. **Insufficient logging**: untraced sensitive actions, secrets in logs.

**C. Sensitive files**
- Permissions, private keys, certificates in the repository.
- Deployment configurations (CI, Docker, cloud): secrets, excessive privileges.

### 3. Report

Report structure:

```markdown
# Security audit — [scope] — [date]

## Summary
- Critical: X · Major: Y · Minor: Z
- Overall verdict: [one sentence]

## Findings

### CRITICAL — [title]
- Where: `file:line`
- Evidence: [code excerpt or command output]
- Impact: [what an attacker can do]
- Fix: [what to change, proposed excerpt]

### MAJOR — [title]
...

### MINOR — [title]
...

## Out of scope / not verified
- [e.g.: "git history not audited", "tests excluded"]
```

### 4. Report rules

- **Every finding has evidence**: file:line + excerpt, or command output. No "there might be a risk".
- **Honest severity**: do not inflate, do not downplay. A committed secret = CRITICAL, not MAJOR.
- **Concrete fix**: every critical/major finding has a proposed corrected code excerpt.
- **No fix in this skill**: the audit produces a report. Fixes happen on a dedicated branch, with `tdd` if relevant.

## Final checklist

- [ ] Secrets/credentials scanned.
- [ ] OWASP Top 10 covered (or exclusions noted).
- [ ] Dependencies audited (command executed, output read).
- [ ] Deployment configs reviewed.
- [ ] Every finding has evidence + severity + fix.
- [ ] Summary with overall verdict.

## Limits to state

- A static audit does not replace a penetration test.
- "Not verified" findings are listed as such.
- Severity reflects potential impact, not probability of exploitation.
